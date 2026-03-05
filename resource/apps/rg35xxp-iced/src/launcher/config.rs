use arc_swap::ArcSwap;
use log::warn;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub volume: u32,
    pub brightness: u32,
    pub screen_timeout: u32,
    pub wallpaper: String,
    pub enable_ssh: bool,
    pub enable_adb: bool,
    pub resource: ResourceConfig,
}
#[derive(Debug, Clone, Serialize, Default, Deserialize)]
#[serde(default)]
pub struct ResourceConfig {
    pub gba: String,
    pub psp: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            volume: 50,
            brightness: 200,
            screen_timeout: 120,
            wallpaper: String::default(),
            enable_ssh: true,
            enable_adb: true,
            resource: Default::default(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct ConfigManager {
    file: PathBuf,
    config: Arc<ArcSwap<Config>>,
    save_channel: std::sync::mpsc::Sender<Config>,
}
fn new_save_channel(file: impl AsRef<Path>) -> std::sync::mpsc::Sender<Config> {
    let file = PathBuf::from(file.as_ref());
    let (sender, receiver) = std::sync::mpsc::channel::<Config>();
    std::thread::spawn(move || {
        while let Ok(config) = receiver.recv() {
            let Ok(cfg) =
                toml::to_string(&config).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
            else {
                continue;
            };
            if let Err(err) = std::fs::write(&file, &cfg) {
                warn!("Failed to save config file: {}", err);
            }
        }
    });
    sender
}
impl ConfigManager {
    pub fn new() -> Self {
        Self {
            file: PathBuf::from("config.toml"),
            save_channel: new_save_channel(PathBuf::from("config.toml")),
            config: Default::default(),
        }
    }
    pub fn load(p: impl AsRef<Path>) -> std::io::Result<Self> {
        let cfg = std::fs::read_to_string(p.as_ref())?;
        let config: Config =
            toml::from_str(&cfg).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Self {
            file: PathBuf::from(p.as_ref()),
            save_channel: new_save_channel(PathBuf::from(p.as_ref())),
            config: Arc::new(ArcSwap::from_pointee(config)),
        })
    }

    pub fn set(&self, f: impl Fn(&mut Config)) {
        let mut config = (**self.config.load()).clone();
        f(&mut config);
        let _ = self.save_channel.send(config.clone());
        self.config.store(Arc::new(config));
    }
    pub fn get_arc(&self) -> Arc<Config> {
        self.config.load_full()
    }
}
