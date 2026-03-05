use std::fmt::{Display, Formatter};
use std::io;
use std::str::FromStr;
use tokio::sync::broadcast::Receiver;

pub type Result<T> = std::result::Result<T, Error>;
pub enum Error {
    Str(&'static str),
    IO(io::Error),
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IO(err)
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Str(s) => {
                write!(f, "{}", s)
            }
            Error::IO(err) => err.fmt(f),
        }
    }
}
pub trait Backlight {
    fn get_brightness(&self) -> Result<u32>;
    /// range 0~100
    fn set_brightness(&self, brightness: u32) -> Result<()>;

    fn backlight_on(&self) -> Result<()>;

    fn backlight_off(&self) -> Result<()>;

    fn get_backlight(&self) -> bool;
}
pub trait Speaker {
    /// range 0~100
    fn get_volume(&self) -> Result<u32>;
    fn set_volume(&self, volume: u32) -> Result<()>;
}

pub trait Backend:
    Backlight + Speaker + Battery + SystemWatcher + Wireless + Power + Service +Clone 
{
}

impl Backlight for () {
    fn get_brightness(&self) -> Result<u32> {
        Ok(100)
    }

    fn set_brightness(&self, _brightness: u32) -> Result<()> {
        Ok(())
    }

    fn backlight_on(&self) -> Result<()> {
        Ok(())
    }

    fn backlight_off(&self) -> Result<()> {
        Ok(())
    }

    fn get_backlight(&self) -> bool {
        true
    }
}

impl Speaker for () {
    fn get_volume(&self) -> Result<u32> {
        Ok(100)
    }

    fn set_volume(&self, _volume: u32) -> Result<()> {
        Ok(())
    }
}

impl Battery for () {
    fn get_battery_capacity(&self) -> Result<u32> {
        Ok(100)
    }

    fn get_battery_status(&self) -> Result<BatteryStatus> {
        Ok(BatteryStatus::Full)
    }
}

impl SystemWatcher for () {
    fn get_subscriber(&self) -> Receiver<Event> {
        todo!()
    }
}

impl Wireless for () {
    fn wifi_scan(&self) -> Result<()> {
        Ok(())
    }

    fn wifi_list(&self) -> Result<Vec<WiFi>> {
        Ok(vec![])
    }

    fn wifi_connect(&self, _ssid: &str, _password: &str) -> Result<()> {
        Ok(())
    }
}

impl Power for () {
    fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    fn restart(&self) -> Result<()> {
        Ok(())
    }
}

impl Service for () {
    fn start(&self, name: impl Into<String>) -> Result<()> {
        Ok(())
    }

    fn stop(&self, name: impl Into<String>) -> Result<()> {
        Ok(())
    }
}

impl Backend for () {}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
}
impl FromStr for BatteryStatus {
    type Err = &'static str;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Charging" => Ok(BatteryStatus::Charging),
            "Discharging" => Ok(BatteryStatus::Discharging),
            "Full" => Ok(BatteryStatus::Full),
            &_ => Err(""),
        }
    }
}
pub trait Battery {
    fn get_battery_capacity(&self) -> Result<u32>;
    fn get_battery_status(&self) -> Result<BatteryStatus>;
}

#[derive(Clone, Debug, Default)]
pub struct WiFi {
    pub ssid: String,
    pub signal: i32,
    pub channel: usize,
    pub security: Vec<String>,
}
impl WiFi {
    /// 将频率 (MHz) 转换为 Wi-Fi 信道
    fn freq_to_channel(freq: usize) -> usize {
        if freq == 2484 {
            14
        } else if freq >= 2407 && freq <= 2472 {
            (freq - 2407) / 5
        } else if freq >= 5170 && freq <= 5825 {
            (freq - 5000) / 5
        } else {
            0 // 未知或不支持
        }
    }

    /// 解析 [WPA2-PSK-CCMP][ESS] 这种格式的字符串
    fn parse_security(flags: &str) -> Vec<String> {
        flags
            .split(']')
            .map(|s| s.trim_start_matches('[').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// 从一行文本解析
    pub fn from_line(line: &str) -> Option<Self> {
        // wpa_cli 的输出是以制表符 \t 分隔的
        let parts: Vec<&str> = line.split('\t').collect();

        // 标准格式至少有 4 列 (bssid, freq, signal, flags)，ssid 可能为空
        if parts.len() < 4 {
            return None;
        }

        let freq = parts[1].parse::<usize>().unwrap_or(0);
        let signal = parts[2].parse::<i32>().unwrap_or(0);
        let security = Self::parse_security(parts[3]);

        // 处理 SSID 可能为空或不存在的情况
        let ssid = parts.get(4).unwrap_or(&"").to_string();

        Some(WiFi {
            ssid,
            signal,
            channel: Self::freq_to_channel(freq),
            security,
        })
    }
}

pub trait Service {
    fn start(&self, name: impl Into<String>) -> Result<()>;
    fn stop(&self, name: impl Into<String>) -> Result<()>;
}
pub trait Power {
    fn shutdown(&self) -> Result<()>;
    fn restart(&self) -> Result<()>;
}
pub trait Wireless {
    /// 发送 wifi 扫描请求
    fn wifi_scan(&self) -> Result<()>;
    /// 查询当前可用 wifi 列表
    fn wifi_list(&self) -> Result<Vec<WiFi>>;
    /// 连接到 wifi
    fn wifi_connect(&self, ssid: &str, password: &str) -> Result<()>;
}

#[derive(Copy, Clone, Debug)]
pub enum Event {
    BacklightOn,
    BacklightOff,
    VolumeChanged(u32),
    BrightnessChanged(u32),
    BatteryCapacityChanged(u32),
    BatteryStatusChanged(BatteryStatus),
}

pub trait SystemWatcher {
    fn get_subscriber(&self) -> Receiver<Event>;
}
