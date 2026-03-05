use crate::launcher;
use crate::launcher::backend::{BatteryStatus, Power, Service, SystemWatcher, WiFi};
use anyhow::anyhow;
use arc_swap::ArcSwap;
use libc::Ioctl;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::os::fd::{AsRawFd, RawFd};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{atomic, Arc};
use std::time::Duration;
use std::{fs, io, thread};
use tokio::net::UdpSocket;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use crate::rg35xxp::ctrl;

// 定义与 C 代码中相同的常量
const DISP_LCD_SET_BRIGHTNESS: Ioctl = 0x102;
const DISP_LCD_GET_BRIGHTNESS: Ioctl = 0x103;
const DISP_LCD_BACKLIGHT_ENABLE: Ioctl = 0x104;
const DISP_LCD_BACKLIGHT_DISABLE: Ioctl = 0x105;
const BRIGHTNESS_MAX: u32 = 255;

enum Instruct {
    WifiScan,
    Shutdown,
    Start(String),
    Stop(String),
    Reboot,
}
pub struct Backlight {
    file: File,
    fd: RawFd,
    channel: u32,
    enabled: atomic::AtomicBool,
}
impl Backlight {
    pub fn new(channel: u32) -> anyhow::Result<Self> {
        // 打开设备文件
        let file = File::options().read(true).write(true).open("/dev/disp")?;
        let fd = file.as_raw_fd();
        Ok(Self {
            file,
            fd,
            channel,
            enabled: atomic::AtomicBool::new(true),
        })
    }

    pub fn set_channel(&mut self, channel: u32) -> anyhow::Result<()> {
        self.channel = channel;
        Ok(())
    }
}
impl crate::launcher::backend::Backlight for Backlight {
    fn get_brightness(&self) -> crate::launcher::backend::Result<u32> {
        // 获取当前亮度
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        let ret = unsafe { libc::ioctl(self.fd, DISP_LCD_GET_BRIGHTNESS, args.as_mut_ptr()) };
        if ret < 0 {
            return Err(io::Error::last_os_error().into());
        }
        self.enabled.store(ret > 0, atomic::Ordering::Relaxed);
        Ok(ret as u32) // 太坑了，直接使用 ret 作为亮度值
    }

    fn set_brightness(&self, brightness: u32) -> crate::launcher::backend::Result<()> {
        if brightness > BRIGHTNESS_MAX {
            return Err(crate::launcher::backend::Error::Str("brightness out of range"));
        }
        // 获取当前亮度
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        args[1] = brightness as libc::c_ulong;
        args[2] = 0;
        unsafe {
            if libc::ioctl(self.fd, DISP_LCD_SET_BRIGHTNESS, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error().into());
            }
        }
        self.enabled
            .store(brightness > 0, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn backlight_on(&self) -> crate::launcher::backend::Result<()> {
        // 打开背光
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        unsafe {
            if libc::ioctl(self.fd, DISP_LCD_BACKLIGHT_ENABLE, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error().into());
            }
        }
        self.enabled.store(true, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn backlight_off(&self) -> crate::launcher::backend::Result<()> {
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        // 关闭背光
        unsafe {
            if libc::ioctl(self.fd, DISP_LCD_BACKLIGHT_DISABLE, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error().into());
            }
        }
        self.enabled.store(false, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn get_backlight(&self) -> bool {
        self.enabled.load(atomic::Ordering::Relaxed)
    }
}

struct Battery {
    capacity: u32,
    status: BatteryStatus,
}

impl Battery {
    pub fn load_axp2202_battery() -> anyhow::Result<Self> {
        let p = Path::new("/sys/class/power_supply/axp2202-battery");
        let capacity: u32 = fs::read_to_string(p.join("capacity"))?.trim().parse()?;
        let status: BatteryStatus = fs::read_to_string(p.join("status"))?
            .trim()
            .parse()
            .map_err(|s| anyhow!("{s}"))?;
        Ok(Self { capacity, status })
    }
}
#[derive(Clone)]
pub(crate) struct RG35xxpBackend(Arc<RG35xxpBackendInner>);
pub(crate) struct RG35xxpBackendInner {
    broadcast: tokio::sync::broadcast::Sender<launcher::backend::Event>,
    battery: Arc<ArcSwap<Battery>>,
    wifi: Arc<ArcSwap<Vec<WiFi>>>,
    backlight: Backlight,
    volume: AtomicU32,
    channel: tokio::sync::mpsc::Sender<Instruct>,
}
impl RG35xxpBackend {
    pub fn new() -> anyhow::Result<Self> {
        let battery = Battery::load_axp2202_battery()?;
        let battery = Arc::new(ArcSwap::new(Arc::new(battery)));
        let backlight = Backlight::new(0)?;
        let battery_arc = battery.clone();
        let (broadcast, _) = tokio::sync::broadcast::channel(1);
        let sender = broadcast.clone();
        let (channel, mut rx) = tokio::sync::mpsc::channel::<Instruct>(1);
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let mut ticket = tokio::time::interval(Duration::from_secs(60));
                let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
                let target: SocketAddr = "127.0.0.1:7788".parse().unwrap();
                loop {
                    select! {
                        _ = ticket.tick() => {
                            match Battery::load_axp2202_battery() {
                                Ok(battery) => {
                                    if battery_arc.load().capacity != battery.capacity {
                                        let _ = sender.send(launcher::backend::Event::BatteryCapacityChanged(battery.capacity));
                                    }
                                    if battery_arc.load().status != battery.status {
                                        let _ = sender.send(launcher::backend::Event::BatteryStatusChanged(battery.status));
                                    }
                                    battery_arc.store(Arc::new(battery));
                                }
                                Err(err) => {
                                    warn!("加载电池信息失败: {}", err);
                                }
                            }
                        }
                        ins = rx.recv() =>{
                            let Some(ins) = ins else{ break};
                            match ins {
                                Instruct::WifiScan => {
                                    let _ = tokio::process::Command::new("wpa_cli")
                                    .arg("-i")
                                    .arg("wlan0")
                                    .arg("scan")
                                    .output().await;
                                }
                                Instruct::Shutdown => {
                                    if let Err(err) = ctrl::call_init::<serde_json::Value>("127.0.0.1:7788",ctrl::Instruct::PowerOff).await{
                                        warn!("shutdown call response error: {}", err);
                                    }
                                }
                                Instruct::Reboot => {
                                    if let Err(err) = ctrl::call_init::<serde_json::Value>("127.0.0.1:7788",ctrl::Instruct::Reboot).await{
                                        warn!("reboot call response error: {}", err);
                                    }
                                }
                                Instruct::Start(name) => {
                                    _ = ctrl::call_init::<serde_json::Value>("127.0.0.1:7788",ctrl::Instruct::StartService {name}).await;
                                }
                                Instruct::Stop(name) => {
                                    _ = ctrl::call_init::<serde_json::Value>("127.0.0.1:7788",ctrl::Instruct::StopService {name}).await;
                                }
                            }
                        }
                    }
                }
            });
        });
        Ok(Self(Arc::new(RG35xxpBackendInner {
            broadcast,
            battery,
            backlight,
            wifi: Arc::new(ArcSwap::new(Arc::new(Vec::new()))),
            volume: AtomicU32::new(50),
            channel,
        })))
    }
}
impl SystemWatcher for RG35xxpBackend {
    fn get_subscriber(&self) -> Receiver<launcher::backend::Event> {
        self.0.broadcast.subscribe()
    }
}

impl crate::launcher::backend::Speaker for RG35xxpBackend {
    fn get_volume(&self) -> crate::launcher::backend::Result<u32> {
        Ok(self.0.volume.load(Ordering::Relaxed))
    }

    fn set_volume(&self, volume: u32) -> crate::launcher::backend::Result<()> {
        if volume > 100 {
            return Err(crate::launcher::backend::Error::Str("volume out of range"));
        }
        info!("setting volume to {}%", volume);
        let volume_str = format!("{}", (volume * 63 + 50) / 100); // 这里不能直接设定 100%，而是转换成为 0-63 之间整数
        let output = Command::new("amixer")
            .arg("cset")
            .arg("name=digital volume") // 注意：直接传字符串，不要加单引号
            .arg(&volume_str)
            .output()?; // output() 会执行命令并捕获输出结果

        if output.status.success() {
            self.0.volume.store(volume, Ordering::Relaxed);
            let _ = self
                .0
                .broadcast
                .send(crate::launcher::backend::Event::VolumeChanged(volume));
            Ok(())
        } else {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            info!("amixer error: {}", err_msg);
            Err(io::Error::new(io::ErrorKind::Other, "amixer 执行失败").into())
        }
    }
}

impl crate::launcher::backend::Backlight for RG35xxpBackend {
    fn get_brightness(&self) -> crate::launcher::backend::Result<u32> {
        self.0.backlight.get_brightness()
    }

    fn set_brightness(&self, brightness: u32) -> crate::launcher::backend::Result<()> {
        self.0.backlight.set_brightness(brightness).and_then(|_| {
            let _ = self
                .0
                .broadcast
                .send(crate::launcher::backend::Event::BrightnessChanged(brightness));
            Ok(())
        })
    }

    fn backlight_on(&self) -> crate::launcher::backend::Result<()> {
        self.0.backlight.backlight_on().and_then(|_| {
            let _ = self
                .0
                .broadcast
                .send(crate::launcher::backend::Event::BacklightOn);
            Ok(())
        })
    }

    fn backlight_off(&self) -> crate::launcher::backend::Result<()> {
        self.0.backlight.backlight_off().and_then(|_| {
            let _ = self
                .0
                .broadcast
                .send(crate::launcher::backend::Event::BacklightOff);
            Ok(())
        })
    }

    fn get_backlight(&self) -> bool {
        self.0.backlight.get_backlight()
    }
}

impl launcher::backend::Battery for RG35xxpBackend {
    fn get_battery_capacity(&self) -> crate::launcher::backend::Result<u32> {
        Ok(self.0.battery.load().capacity)
    }

    fn get_battery_status(&self) -> crate::launcher::backend::Result<BatteryStatus> {
        Ok(self.0.battery.load().status)
    }
}

pub fn remove_network_by_ssid(target_ssid: &str) -> std::io::Result<()> {
    let interface = "wlan0";

    // 1. 获取列表
    let output = Command::new("wpa_cli")
        .args(&["-i", interface, "list_networks"])
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // 2. 解析 ID
    for line in stdout.lines().skip(1) {
        // 跳过标题行
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let id = parts[0].trim();
            let ssid = parts[1].trim();

            if ssid == target_ssid {
                // 3. 找到匹配的 SSID，调用删除
                let output = Command::new("wpa_cli")
                    .args(&["-i", interface, "remove_network", id])
                    .output()?;
                let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return if response == "OK" || response.contains("network id") {
                    Ok(())
                } else {
                    Err(Error::new(
                        ErrorKind::Other,
                        format!("wpa_cli error: {}", response),
                    ))
                };
            }
        }
    }

    Err(Error::new(
        ErrorKind::NotFound,
        "SSID not found in saved networks",
    ))
}
impl launcher::backend::Wireless for RG35xxpBackend {
    fn wifi_scan(&self) -> launcher::backend::Result<()> {
        let _ = self.0.channel.blocking_send(Instruct::WifiScan);
        Ok(())
    }

    fn wifi_list(&self) -> launcher::backend::Result<Vec<WiFi>> {
        let output = std::process::Command::new("wpa_cli")
            .arg("-i")
            .arg("wlan0")
            .arg("scan_result")
            .output()?;
        let mut wifi_list = Vec::new();
        for line in String::from_utf8_lossy(output.stdout.as_slice())
            .lines()
            .skip(1)
        {
            if let Some(wifi) = WiFi::from_line(line) {
                wifi_list.push(wifi);
            }
        }
        Ok(wifi_list)
    }

    fn wifi_connect(&self, ssid: &str, password: &str) -> launcher::backend::Result<()> {
        let interface = "wlan0";
        let net_id = "0";

        fn to_hex(s: &str) -> String {
            s.as_bytes().iter().map(|b| format!("{:02x}", b)).collect()
        }

        let run_wpa_cmd = |args: &[&str]| -> std::io::Result<()> {
            info!("run_wpa_cmd: wpa_cli -i {} {}", interface, args.join(" "));
            let output = Command::new("wpa_cli")
                .arg("-i")
                .arg(interface)
                .args(args)
                .output()?;

            let response = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if response == "OK" || response.contains("Selected network") {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorKind::Other,
                    format!("wpa_cli error: {}", response),
                ))
            }
        };

        let _ = run_wpa_cmd(&["remove_network", "all"]);
        let _ = run_wpa_cmd(&["add_network"]);

        // 1. 将 SSID 转换为十六进制
        // 注意：这里没有引号！
        let ssid_hex = to_hex(ssid);
        run_wpa_cmd(&["set_network", net_id, "ssid", &ssid_hex])?;

        // 2. 将密码转换为十六进制
        // 注意：这里也没有引号！
        run_wpa_cmd(&["set_network", net_id, "psk", &format!("\"{}\"", password)])?;

        // 3. 启用并选择网络
        run_wpa_cmd(&["enable_network", net_id])?;
        run_wpa_cmd(&["select_network", net_id])?;
        let _ = run_wpa_cmd(&["save_config"]);

        info!("Connecting to {} (Hex: {})", ssid, ssid_hex);
        Ok(())
    }
}
#[derive(Deserialize, Serialize, Debug)]
enum InitInstruct {
    Reboot,
    PowerOff,
    ListService,
    RestartService { name: String },
    StopService { name: String },
    StartService { name: String },
}

impl Power for RG35xxpBackend {
    fn shutdown(&self) -> launcher::backend::Result<()> {
        let _ = self.0.channel.blocking_send(Instruct::Shutdown);
        Ok(())
    }

    fn restart(&self) -> launcher::backend::Result<()> {
        let _ = self.0.channel.blocking_send(Instruct::Reboot);
        Ok(())
    }
}

impl Service for RG35xxpBackend {
    fn start(&self, name: impl Into<String>) -> launcher::backend::Result<()> {
        let _ = self.0.channel.blocking_send(Instruct::Start(name.into()));
        Ok(())
    }

    fn stop(&self, name: impl Into<String>) -> launcher::backend::Result<()> {
        let _ = self.0.channel.blocking_send(Instruct::Stop(name.into()));
        Ok(())
    }
}

impl launcher::backend::Backend for RG35xxpBackend {}
