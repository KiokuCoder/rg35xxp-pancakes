use crate::launcher;
use crate::launcher::launcher::{LauncherContext, Message};
use crate::launcher::pad::DPad;
pub(crate) mod game;
pub(crate) mod software;
pub(crate) mod wifi;
pub(crate) mod file_manager;
pub(crate) mod settings;

use crate::launcher::page::file_manager::FileManager;
use crate::launcher::page::game::GameList;
use crate::launcher::page::settings::Settings;
use crate::launcher::page::software::SoftwareList;
use crate::launcher::page::wifi::WifiPage;

pub(crate) enum Page {
    GameList(GameList),
    SoftwareList(SoftwareList),
    Settings(Settings),
    Wifi(WifiPage),
    FileManager(FileManager),
}
impl Page {
    pub fn title(&self) -> &'static str {
        match self {
            Page::GameList(_) => "Games",
            Page::SoftwareList(_) => "Software",
            Page::Settings(_) => "Settings",
            Page::Wifi(_) => "WiFi",
            Page::FileManager(_) => "File Manager",
        }
    }
    pub fn view(&self) -> launcher::Element<'_> {
        match self {
            Page::GameList(game_list) => game_list.view(),
            Page::SoftwareList(software) => software.view(),
            Page::Settings(settings) => settings.view(),
            Page::Wifi(wifi) => wifi.view(),
            Page::FileManager(fm) => fm.view(),
        }
    }
    /// 返回 true 表示页面关闭
    pub fn handle(&mut self, rt: impl LauncherContext, key: DPad) -> bool {
        match self {
            Page::GameList(game_list) => game_list.handle(rt, key),
            Page::SoftwareList(software_list) => software_list.handle(rt, key),
            Page::Settings(settings) => settings.handle(rt, key),
            Page::Wifi(wifi) => return wifi.handle(rt, key),
            Page::FileManager(fm) => return fm.handle(rt, key),
        };
        return false;
    }
    pub fn update(&mut self, rt: impl LauncherContext, message: Message) {
        match self {
            Page::GameList(_) => {}
            Page::SoftwareList(_) => {}
            Page::Settings(p) => {
                if let Message::Settings(msg) = message {
                    p.update(msg);
                }
            }
            Page::Wifi(_) => {}
            Page::FileManager(_) => {}
        }
    }

    pub fn save(&self) -> PageDescription {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct PageDescription {
    pub name: String,
    pub state: serde_json::Value,
}

