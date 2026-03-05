use crate::launcher;
use crate::launcher::backend::Backend;
use crate::launcher::config::ConfigManager;
use crate::launcher::executor::Push;
use crate::launcher::pad::DPad;
use crate::launcher::page::{file_manager, settings, Page};
use crate::launcher::{fonts};
use iced_core::keyboard::{Event, Key};
use iced_core::{alignment, keyboard, Background, Color, Length};
use iced_widget::{container, row, space, text};
use image::{imageops};
use image::imageops::FilterType;
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};
use crate::launcher::page::game::GameList;
use crate::launcher::page::settings::SettingMessage;
use crate::launcher::page::software::SoftwareList;
use crate::launcher::ui::alert::{Alert, ShowAlert};
use crate::launcher::ui::WALLPAPER;

#[derive(Debug, Clone)]
pub enum Message {
    Batch(Vec<Message>),
    Noop,
    Refresh,
    Pad(DPad),
    KeyEvent(keyboard::Event),
    Launch {
        exec: String,
        args: Vec<String>,
        wd:String,
    },
    SetVolume(u32),
    SetBrightness(u32),
    SetTimeout(u32), // 设定超时时间
    SetWallpaper(String),
    Enable(&'static str),
    Disable(&'static str),
    Ticker,          // 300 帧一次
    Screenshot,          // 截图指令
    ScanWifi,
    Shutdown,
    Reboot,
    ScanWifiFinished,
    Settings(SettingMessage),
    ConnectWifi { ssid: String, psk: String },
    Alert(Alert),
}

pub trait LauncherContext: ShowAlert + Push<Message> {}
impl<T> LauncherContext for T where T: ShowAlert + Push<Message> {}

pub(crate) struct Launcher<B: Backend> {
    pub(crate) backend: B,
    pub(crate) volume_last_update: Instant, // 用来显示浮动提示
    pub(crate) last_input: Instant,         // 用来处理自动休眠
    pub(crate) network: u32,                // 网络强度
    pub(crate) key_map: HashMap<String, DPad>, // 按键映射
    pub(crate) wallpaper: iced::advanced::image::Handle,
    pub(crate) pages: Vec<Page>,
    pub(crate) pages_index: usize,
    pub(crate) time: String,
    pub(crate) battery: String,
    pub(crate) config: ConfigManager,
    pub(crate) alert: Option<Alert>,
}
impl<B: Backend + 'static + Send> Launcher<B> {
    pub fn new(backend: B) -> anyhow::Result<Self> {
        fonts::load();
        let config = ConfigManager::load(env::var("CONFIG_FILE").unwrap_or("config.toml".to_string()))
            .unwrap_or(ConfigManager::new());
        let cfg = config.get_arc();
        let gba_dir = if cfg.resource.gba.is_empty() {
            "/mnt/mmc/GBA"
        } else {
            cfg.resource.gba.as_str()
        };
        let pages = vec![
            Page::GameList(GameList::load(gba_dir).unwrap_or(GameList {
                games: vec![],
                offset: 0,
                selected: 0,
            })),
            Page::SoftwareList(SoftwareList::load("apps")?),
            Page::Settings(settings::Settings::new(backend.clone(),config.clone())?),
        ];
        let t = chrono::Local::now();

        let result = Self {
            config,
            backend,
            wallpaper:Self::wallpaper(cfg.wallpaper.as_str()),
            volume_last_update: Instant::now() - Duration::from_secs(10),
            last_input: Instant::now(),
            network: 0,
            key_map: Default::default(),
            pages,
            pages_index: 0,
            time: t.format("%H:%M").to_string(),
            battery: "0 %".to_string(),
            alert: None,
        };
        result.init();
        Ok(result)
    }

    pub fn view(&self) -> launcher::Element<'_> {
        let title = self
            .pages
            .get(self.pages_index)
            .map(|p| p.title())
            .unwrap_or("");

        let battery_icon = if let Ok(status) = self.backend.get_battery_status() {
            match status {
                crate::launcher::backend::BatteryStatus::Charging => "battery_charging_full",
                crate::launcher::backend::BatteryStatus::Discharging => {
                    let capacity = self.backend.get_battery_capacity().unwrap_or(100);
                    if capacity <= 15 {
                        "battery_alert"
                    } else if capacity <= 30 {
                        "battery_2_bar"
                    } else if capacity <= 60 {
                        "battery_4_bar"
                    } else {
                        "battery_full"
                    }
                }
                crate::launcher::backend::BatteryStatus::Full => "battery_full",
            }
        } else {
            "battery_full"
        };

        let icon_style = launcher::ui::icon::outlined().size(32).color(Color::WHITE);

        let mut right_items = row![].spacing(10).align_y(alignment::Vertical::Center);
        if self.network > 0 {
            right_items = right_items.push(icon_style.icon("wifi"));
        }
        right_items = right_items.push(
            row![
                icon_style.icon(battery_icon),
                text(&self.battery).size(24)
            ]
            .spacing(5)
            .align_y(alignment::Vertical::Center),
        );

        let topbar = container(
            row![
                container(text(title).size(24)).padding([0, 20]).center_y(Length::Fill),
                container(space()).width(Length::Fill),
                row![
                    icon_style.icon("schedule"),
                    text(&self.time).size(24)
                ]
                .spacing(5)
                .align_y(alignment::Vertical::Center),
                container(space()).width(Length::Fill),
                container(right_items).padding([0, 20]).center_y(Length::Fill),
            ]
            .align_y(alignment::Vertical::Center),
        )
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba8(100, 100, 100, 0.5))),
            text_color: Some(Color::WHITE),
            ..Default::default()
        })
        .center_y(60)
        .width(Length::Fill);
        let empty = container(text("System Error!"));
        let content = self
            .pages
            .get(self.pages_index)
            .map(|p| p.view())
            .unwrap_or(empty.into());

        let mut stack = iced_widget::stack![
            iced_widget::image(&self.wallpaper),
            iced_widget::column![topbar, content]
        ];

        if let Some(alert) = &self.alert {
            stack = stack.push(alert.view());
        }

        stack.into()
    }

    pub fn update(&mut self, rt: impl LauncherContext, message: Message) {
        match message {
            Message::Batch(messages) => {
                for message in messages {
                    self.update(rt.clone(), message);
                }
            }
            Message::Pad(k) => {
                self.last_input = Instant::now();
                if self.alert.is_some() {
                    if let DPad::A = k {
                        self.alert = None;
                    } else if let DPad::B = k {
                        self.alert = None;
                    }
                    return;
                }
                match k {
                    DPad::Power => {}
                    DPad::L1 => {
                        self.pages_index =
                            (self.pages_index + self.pages.len() - 1) % self.pages.len();
                    }
                    DPad::R1 => {
                        self.pages_index = (self.pages_index + 1) % self.pages.len();
                    }
                    DPad::VolumeUp => {
                        let volume = self.backend.get_volume().unwrap_or_default();
                        if volume <= 90 {
                            let _ = self.backend.set_volume(volume + 10);
                        }
                    }
                    DPad::VolumeDown => {
                        info!("volume down");
                        let volume = self.backend.get_volume().unwrap_or_default();
                        if volume >= 10 {
                            let _ = self.backend.set_volume(volume - 10);
                        }
                    }
                    _ => {
                        self.pages[self.pages_index].handle(rt, k);
                    }
                }
            }
            Message::Noop => {}
            Message::KeyEvent(Event::KeyReleased {
                key: Key::Named(name),
                ..
            }) => {
                if let Some(k) = self.key_map.get(&format!("{:?}", name)) {
                    rt.push(Message::Pad(k.clone()));
                }
            }
            Message::KeyEvent(Event::KeyReleased {
                key: Key::Character(c),
                ..
            }) => {
                if let Some(k) = self.key_map.get(c.as_str()) {
                    rt.push(Message::Pad(k.clone()));
                }
            }
            Message::KeyEvent(_k) => {}
            Message::Launch { exec, .. } => {
                info!("launching {:?}", exec);
            }
            Message::SetVolume(v) => {
                if let Err(err) = self.backend.set_volume(v) {
                    error!("设定音量失败: {}", err);
                }
                // 更新配置
                self.config.set(|cfg| {
                    cfg.volume = v;
                });
            }
            Message::SetBrightness(v) => {
                info!("set brightness to {}", v);
                if let Err(err) = self.backend.set_brightness(v) {
                    error!("设定亮度失败: {}", err);
                }
                self.config.set(|cfg| {
                    cfg.brightness = v;
                });
            }
            Message::Ticker => {
                self.time = chrono::Local::now().format("%H:%M").to_string();
                self.battery = format!(
                    "{} %",
                    self.backend.get_battery_capacity().unwrap_or_default()
                );
            }
            Message::SetTimeout(v) => {
                self.config.set(|cfg| {
                    cfg.screen_timeout = v;
                });
            }
            Message::ScanWifi => {
                info!("Scanning wifi...");
                if let Err(err) = self.backend.wifi_scan() {
                    error!("{}", err);
                }
                rt.spawn(async {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    Message::ScanWifiFinished
                });
            }
            Message::ScanWifiFinished => {
                let networks = self.backend.wifi_list().unwrap_or_default();
                info!("Wifi scan finished: {} networks found", networks.len());
                for page in &mut self.pages {
                    if let Page::Settings(settings) = page {
                        settings.update_networks(networks);
                        break;
                    }
                }
            }
            Message::Alert(alert) => {
                self.alert = Some(alert);
            }
            Message::ConnectWifi { ssid, psk } => {
                if let Err(err) = self.backend.wifi_connect(&ssid, &psk) {
                    error!("wifi connect error: {}", err);
                }
            }
            Message::Settings(msg) => {
                for page in &mut self.pages {
                    if let Page::Settings(settings) = page {
                        settings.update(msg);
                        break;
                    }
                }
            }
            Message::Shutdown => {
                let _ = self.backend.shutdown();
            }
            Message::Reboot => {
                let _ = self.backend.restart();
            }
            Message::SetWallpaper(p) => {
                self.wallpaper = Self::wallpaper(p.as_str());
                self.config.set(|cfg| {
                    cfg.wallpaper = p.clone();
                });
            }
            Message::Refresh => {}
            Message::Enable(service) => {
                if service == "ssh" {
                    self.config.set(|cfg|{
                        cfg.enable_ssh = true;
                    });
                    _ = self.backend.start("dropbear");
                }
                if service == "adb" {
                    self.config.set(|cfg|{
                        cfg.enable_adb = true;
                    });
                    _ = self.backend.start("adb");
                }
            }
            Message::Disable(service) => {
                if service == "ssh" {
                    self.config.set(|cfg|{
                        cfg.enable_ssh = false;
                    });
                    _ = self.backend.stop("dropbear");
                }
                if service == "adb" {
                    self.config.set(|cfg|{
                        cfg.enable_adb = false;
                    });
                    _ = self.backend.stop("adb");
                }
            }
            Message::Screenshot => {}
        }
    }

    // 同步配置到后端
    fn init(&self) {
        let cfg = self.config.get_arc();
        let _ = self.backend.backlight_on();
        let _ = self.backend.set_volume(cfg.volume);
        let _ = self.backend.set_brightness(cfg.brightness);
    }

    // 设定壁纸
    fn wallpaper(p: &str) -> iced::advanced::image::Handle {
        let w = if !p.is_empty() {
            image::open(p).ok()
        } else {
            None
        };
        let p = match w {
            None => { image::load_from_memory(WALLPAPER).unwrap() }
            Some(w) => { w }
        };
        // iced 显示图片超过 1080p 会爆显存，导致无法正常显示，具体大小未测试
        // 强制缩放
        let mut p = p.resize(640, 480, FilterType::Gaussian);
        let region = p.crop_imm(0, 0, 640, 60).blur(5.0);
        imageops::replace(&mut p, &region, 0, 0);
        iced::advanced::image::Handle::from_rgba(
            p.width(),
            p.height(),
            p.to_rgba8().into_raw(),
        )
    }

}
