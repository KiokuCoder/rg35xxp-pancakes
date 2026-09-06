use libc::Ioctl;
use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};
use std::sync::atomic;
use std::time::Duration;
use std::io;

use evdev::{EventType, KeyCode};
use std::env;
use std::process::{ExitCode, ExitStatus, Stdio};
use tokio::process::Command;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    // Check arguments
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        return ExitCode::FAILURE;
    }
    let backlight = match Backlight::new(0) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to open backlight device (/dev/disp): {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Target command and arguments
    let target_command = &args[1];
    let target_args = &args[2..];

    // Spawn child process
    let mut child = match Command::new(target_command)
        .args(target_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Failed to start command '{}': {}", target_command, e);
            return ExitCode::FAILURE;
        }
    };
    
    let pid = match child.id() {
        Some(id) => id,
        None => {
            eprintln!("Failed to get child process PID");
            return ExitCode::FAILURE;
        }
    };

    // 电源键取第一个支持 KEY_POWER 的设备。音量键不一定在同一个设备上，
    // 落在别的设备上就再开一个流。两种键都不 grab，事件照样会传给子进程。
    let mut power_device = None;
    let mut volume_devices = Vec::new();
    for (_, device) in evdev::enumerate() {
        let (is_power, is_volume) = match device.supported_keys() {
            Some(keys) => (
                keys.contains(KeyCode::KEY_POWER),
                keys.contains(KeyCode::KEY_VOLUMEUP) || keys.contains(KeyCode::KEY_VOLUMEDOWN),
            ),
            None => (false, false),
        };
        if is_power && power_device.is_none() {
            power_device = Some(device);
        } else if is_volume {
            volume_devices.push(device);
        }
    }
    let Some(power_device) = power_device else {
        eprintln!("Failed to find power button device");
        return ExitCode::FAILURE;
    };

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1);
    
    let tx_exit = tx.clone();
    tokio::spawn(async move {
        if let Ok(status) = child.wait().await {
            let _ = tx_exit.send(Event::Exit(status)).await;
        }
    });

    if let Err(e) = spawn_key_reader(power_device, tx.clone()) {
        eprintln!("Failed to listen power button device: {}", e);
        return ExitCode::FAILURE;
    }
    // 没有音量键设备不算致命错误，顶多是调不了音量
    if volume_devices.is_empty() {
        eprintln!("No volume key device found, volume keys disabled");
    }
    for device in volume_devices {
        if let Err(e) = spawn_key_reader(device, tx.clone()) {
            eprintln!("Failed to listen volume key device: {}", e);
        }
    }

    let tx_ctrlc = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(_) = tokio::signal::ctrl_c().await {
                if let Err(_) = tx_ctrlc.send(Event::CtrlC).await {
                    break;
                }
            } else {
                break;
            }
        }
    });

    // Get sleep timeout from environment or default to 300s
    let sleep_timeout = env::var("SLEEP_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300);

    let mut paused = false;
    let mut paused_gen = 0;
    let mut last_wake = std::time::Instant::now() - Duration::from_secs(1);
    let mut volume = Volume::new();
    let mut last_volume = std::time::Instant::now() - Duration::from_secs(1);
    while let Some(event) = rx.recv().await {
        match event {
            Event::Power => {
                if last_wake.elapsed() < Duration::from_millis(500) {
                    continue;
                }
                if paused {
                    paused = false;
                    paused_gen += 1;
                    let _ = backlight.backlight_on();
                    let _ = resume(pid);
                } else {
                    paused = true;
                    paused_gen += 1;
                    let current_gen = paused_gen;
                    let tx_timeout = tx.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(sleep_timeout)).await;
                        let _ = tx_timeout.send(Event::Timeout(current_gen)).await;
                    });
                    let _ = pause(pid);
                    let _ = backlight.backlight_off();
                }
            }
            Event::Exit(status) => {
                return if let Some(code) = status.code() {
                    ExitCode::from(code as u8)
                } else {
                    ExitCode::FAILURE
                };
            }
            Event::Timeout(g) => {
                if !paused || g != paused_gen {
                    continue;
                }
                // Suspend system, blocks until wake
                if let Err(e) = std::fs::write("/sys/power/state", "mem") {
                    eprintln!("Failed to write /sys/power/state: {}", e);
                }
                // System waked up
                last_wake = std::time::Instant::now();
                paused = false;
                paused_gen += 1;
                let _ = backlight.backlight_on();
                let _ = resume(pid);
            }
            Event::Volume(up) => {
                // 按住不放时内核的自动重复也会走到这里，限一下速，
                // 免得一秒钟 fork 几十个 amixer
                if last_volume.elapsed() < Duration::from_millis(100) {
                    continue;
                }
                last_volume = std::time::Instant::now();
                volume.step(up).await;
            }
            Event::CtrlC => {
                // Forward SIGINT to child
                let _ = resume(pid);
                unsafe { libc::kill(pid as i32, libc::SIGINT) };
            }
        }
    }
    return ExitCode::FAILURE;
}
enum Event {
    Power,
    Volume(bool), // true=调大
    Exit(ExitStatus),
    Timeout(u32),
    CtrlC,
}

fn spawn_key_reader(device: evdev::Device, tx: tokio::sync::mpsc::Sender<Event>) -> io::Result<()> {
    device.set_nonblocking(true)?;
    let mut stream = device.into_event_stream()?;
    tokio::spawn(async move {
        loop {
            let ev = match stream.next_event().await {
                Ok(ev) => ev,
                Err(e) => {
                    eprintln!("Failed to read input device: {}", e);
                    break;
                }
            };
            if ev.event_type() != EventType::KEY {
                continue;
            }
            // 电源键沿用原来的逻辑，在抬起时触发；音量键按下就调一档，
            // 按住不放时内核的自动重复(value==2)也算一档
            let event = match (ev.code(), ev.value()) {
                (c, 0) if c == KeyCode::KEY_POWER.0 => Event::Power,
                (c, 1 | 2) if c == KeyCode::KEY_VOLUMEUP.0 => Event::Volume(true),
                (c, 1 | 2) if c == KeyCode::KEY_VOLUMEDOWN.0 => Event::Volume(false),
                _ => continue,
            };
            if tx.send(event).await.is_err() {
                break;
            }
        }
    });
    Ok(())
}

// 系统音量。对应 alsa 的 "digital volume"(0-63)，和 rg35xxp-launcher 设置页
// 调的是同一个控件，所以这里也按 10% 一档走，两边调出来的值能对上。
const MIXER_CONTROL: &str = "name=digital volume";
const VOLUME_STEP: u32 = 10;

struct Volume {
    // (当前百分比, alsa 控件的最大值)。第一次按音量键时从 alsa 读一次，之后只在
    // 这里按档加减：alsa 那边只有 0-63 档，每次都换算回百分比会掉精度，
    // 会出现按了一下没反应的空档。
    state: Option<(u32, u32)>,
    // 没装 amixer 或者没有这个控件，报一次就不再重试
    unavailable: bool,
}

impl Volume {
    fn new() -> Self {
        Self { state: None, unavailable: false }
    }

    async fn step(&mut self, up: bool) {
        if self.unavailable {
            return;
        }
        let (percent, max) = match self.state {
            Some(v) => v,
            None => match read_volume().await {
                Ok((raw, max)) => (snap_percent(to_percent(raw, max)), max),
                Err(e) => {
                    eprintln!("Failed to read alsa volume: {}", e);
                    self.unavailable = true;
                    return;
                }
            },
        };
        let next = step_percent(percent, up);
        // 顶到两端时也照写一次：alsa 的档位比 10% 细，state 里记的百分比
        // 未必和 alsa 里的值严格对应，写一次才能真的推到 0 或者最大
        if let Err(e) = write_volume(to_raw(next, max)).await {
            eprintln!("Failed to set alsa volume: {}", e);
            return;
        }
        self.state = Some((next, max));
        println!("volume: {}%", next);
    }
}

fn to_percent(raw: u32, max: u32) -> u32 {
    (raw * 100 + max / 2) / max
}

fn to_raw(percent: u32, max: u32) -> u32 {
    (percent * max + 50) / 100
}

// alsa 的档位(0-63)比 10% 细，从 alsa 读回来的值先归到最近的一档上，
// 否则第一次按下去可能算回同一个 alsa 值，表现成按了没反应
fn snap_percent(percent: u32) -> u32 {
    (percent + VOLUME_STEP / 2) / VOLUME_STEP * VOLUME_STEP
}

// 跳到相邻的那一档，按住不放就一档一档走
fn step_percent(percent: u32, up: bool) -> u32 {
    if up {
        (percent / VOLUME_STEP * VOLUME_STEP + VOLUME_STEP).min(100)
    } else {
        percent.div_ceil(VOLUME_STEP).saturating_sub(1) * VOLUME_STEP
    }
}

async fn read_volume() -> io::Result<(u32, u32)> {
    let output = Command::new("amixer").arg("cget").arg(MIXER_CONTROL).output().await?;
    if !output.status.success() {
        return Err(io::Error::other(String::from_utf8_lossy(&output.stderr).trim().to_string()));
    }
    parse_cget(&String::from_utf8_lossy(&output.stdout))
}

// amixer cget 的输出形如：
//   numid=8,iface=MIXER,name='digital volume'
//     ; type=INTEGER,access=rw---R--,values=1,min=0,max=63,step=0
//     : values=40
// 注意 "values=" 在类型那一行也出现过，所以当前值只认带冒号的那一行。
fn parse_cget(text: &str) -> io::Result<(u32, u32)> {
    let max = text
        .split("max=")
        .nth(1)
        .and_then(|s| s.split(|c: char| !c.is_ascii_digit()).next())
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|max| *max > 0)
        .ok_or_else(|| io::Error::other("no max= in amixer output"))?;
    let current = text
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(": values="))
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<u32>().ok())
        .ok_or_else(|| io::Error::other("no values= in amixer output"))?;
    Ok((current.min(max), max))
}

async fn write_volume(raw: u32) -> io::Result<()> {
    // 注意：name 直接传字符串，不要加单引号
    let output = Command::new("amixer")
        .arg("cset")
        .arg(MIXER_CONTROL)
        .arg(raw.to_string())
        .output()
        .await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(String::from_utf8_lossy(&output.stderr).trim().to_string()))
    }
}
const _DISP_LCD_SET_BRIGHTNESS: Ioctl = 0x102;
const _DISP_LCD_GET_BRIGHTNESS: Ioctl = 0x103;
const DISP_LCD_BACKLIGHT_ENABLE: Ioctl = 0x104;
const DISP_LCD_BACKLIGHT_DISABLE: Ioctl = 0x105;
const _BRIGHTNESS_MAX: u32 = 255;
pub struct Backlight {
    _file: File,
    fd: RawFd,
    channel: u32,
    enabled: atomic::AtomicBool,
}
impl Backlight {
    fn new(channel: u32) -> std::io::Result<Self> {
        let file = File::options().read(true).write(true).open("/dev/disp")?;
        let fd = file.as_raw_fd();
        Ok(Self {
            _file: file,
            fd,
            channel,
            enabled: atomic::AtomicBool::new(true),
        })
    }

    fn _set_channel(&mut self, channel: u32) -> std::io::Result<()> {
        self.channel = channel;
        Ok(())
    }

    fn _get_brightness(&self) -> std::io::Result<u32> {
        // Get current brightness
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        let ret = unsafe { libc::ioctl(self.fd, _DISP_LCD_GET_BRIGHTNESS, args.as_mut_ptr()) };
        if ret < 0 {
            return Err(io::Error::last_os_error().into());
        }
        self.enabled.store(ret > 0, atomic::Ordering::Relaxed);
        Ok(ret as u32) // Use ret directly as brightness value
    }

    fn _set_brightness(&self, brightness: u32) -> std::io::Result<()> {
        if brightness > _BRIGHTNESS_MAX {
            return Err(std::io::Error::other("brightness out of range"));
        }
        // Set brightness
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        args[1] = brightness as libc::c_ulong;
        args[2] = 0;
        unsafe {
            if libc::ioctl(self.fd, _DISP_LCD_SET_BRIGHTNESS, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error().into());
            }
        }
        self.enabled
            .store(brightness > 0, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn backlight_on(&self) -> std::io::Result<()> {
        // Enable backlight
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        unsafe {
            if libc::ioctl(self.fd, DISP_LCD_BACKLIGHT_ENABLE, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        self.enabled.store(true, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn backlight_off(&self) -> std::io::Result<()> {
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        // Disable backlight
        unsafe {
            if libc::ioctl(self.fd, DISP_LCD_BACKLIGHT_DISABLE, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        self.enabled.store(false, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn _get_backlight(&self) -> bool {
        self.enabled.load(atomic::Ordering::Relaxed)
    }
}

fn pause(pid: u32) -> std::io::Result<()> {
    if unsafe { libc::kill(pid as i32, libc::SIGSTOP) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
fn resume(pid: u32) -> std::io::Result<()> {
    if unsafe { libc::kill(pid as i32, libc::SIGCONT) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CGET: &str = "numid=8,iface=MIXER,name='digital volume'\n  ; type=INTEGER,access=rw---R--,values=1,min=0,max=63,step=0\n  : values=40\n";

    #[test]
    fn parse_cget_reads_current_and_max() {
        assert_eq!(parse_cget(CGET).unwrap(), (40, 63));
        // 当前值超出 max 时按 max 算，别让后面的百分比算出 100 以上
        assert!(parse_cget("; max=63\n: values=99").is_ok_and(|v| v == (63, 63)));
        assert!(parse_cget("").is_err());
        assert!(parse_cget("; min=0,max=0\n: values=0").is_err());
        assert!(parse_cget("; min=0,max=63").is_err());
    }

    #[test]
    fn percent_matches_launcher_formula() {
        // rg35xxp-launcher 用的是 (volume * 63 + 50) / 100，两边必须一致
        for percent in (0..=100).step_by(10) {
            assert_eq!(to_raw(percent, 63), (percent * 63 + 50) / 100);
        }
        assert_eq!(to_raw(0, 63), 0);
        assert_eq!(to_raw(100, 63), 63);
    }

    #[test]
    fn every_press_moves_the_alsa_value() {
        // 不管 alsa 里原来是哪一档，第一次按下去都得真的动一格，
        // 中间不能出现按了没反应的空档(除非已经顶到两端)
        for raw in 0..=63 {
            let percent = snap_percent(to_percent(raw, 63));
            let up = step_percent(percent, true);
            if up != percent {
                assert!(to_raw(up, 63) > raw, "raw={} percent={} up={}", raw, percent, up);
            }
            let down = step_percent(percent, false);
            if down != percent {
                assert!(to_raw(down, 63) < raw, "raw={} percent={} down={}", raw, percent, down);
            }
        }
    }

    #[test]
    fn snap_percent_rounds_to_the_nearest_grid_point() {
        assert_eq!(snap_percent(21), 20);
        assert_eq!(snap_percent(25), 30);
        assert_eq!(snap_percent(0), 0);
        assert_eq!(snap_percent(98), 100);
        assert_eq!(snap_percent(100), 100);
    }

    #[test]
    fn step_percent_walks_the_grid() {
        assert_eq!(step_percent(40, true), 50);
        assert_eq!(step_percent(40, false), 30);
        // 不在档位上时走到相邻的那一档，不会原地不动
        assert_eq!(step_percent(44, true), 50);
        assert_eq!(step_percent(44, false), 40);
        // 两端夹住
        assert_eq!(step_percent(100, true), 100);
        assert_eq!(step_percent(95, true), 100);
        assert_eq!(step_percent(0, false), 0);
        assert_eq!(step_percent(5, false), 0);
        assert_eq!(step_percent(0, true), 10);
        assert_eq!(step_percent(100, false), 90);
    }
}
