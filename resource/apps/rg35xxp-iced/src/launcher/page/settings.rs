use crate::launcher;
use crate::launcher::backend::{Backend, SystemWatcher, WiFi};
use crate::launcher::ui::dialog::Confirm;
use crate::launcher::ui::form::{Form, Select, Text};
use crate::launcher::launcher::{LauncherContext, Message};
use crate::launcher::pad::DPad;
use crate::launcher::page::{Page, PageDescription};
use crate::launcher::ui::{dialog, form};
use arc_swap::ArcSwap;
use std::sync::Arc;
use crate::launcher::config::ConfigManager;
use crate::launcher::page::file_manager::FileManager;
use crate::launcher::page::wifi::WifiPage;
use crate::launcher::ui::toolkit::indicator;

#[derive(Debug, Clone)]
pub enum SettingMessage {
    ShowShutdownConfirm,
    ShowRebootConfirm,
    NewPage(PageDescription),
}
pub(crate) struct Settings {
    inner: Form<Message, &'static str>,
    pages: Vec<Page>,
    dialog: Option<Confirm>,
}
#[derive(Debug, Clone)]
struct StateInner {
    volume: u32,
    brightness: u32,
}

impl Settings {
    pub fn new(backend: impl Backend + SystemWatcher,config:ConfigManager) -> anyhow::Result<Self> {
        let volume = backend.get_volume().unwrap_or(50);
        let brightness = backend.get_brightness().unwrap_or(128);
        let state: Arc<ArcSwap<StateInner>> =
            Arc::new(ArcSwap::from_pointee(StateInner { volume, brightness }));

        let volume: Select<Message, &_> = Select {
            name: "Volume".to_string(),
            value: volume_to_index(state.load().volume),
            options: vec!["0", "1", "2", "3", "4"],
            handle: None,
        }
        .on_change(|v| {
            let i: usize = v.parse().unwrap();
            Message::SetVolume(volume_from_index(i))
        });
        let brightness: Select<Message, &_> = Select {
            name: "Brightness".to_string(),
            value: brightness_to_index(state.load().brightness),
            options: vec!["0", "1", "2", "3", "4"],
            handle: None,
        }
        .on_change(|v| {
            let i: usize = v.parse().unwrap();
            Message::SetBrightness(brightness_from_index(i))
        });
        let cfg = config.get_arc();
        let adb = form::Check {
            name: "Enable adb".to_string(),
            value:cfg.enable_adb ,
            handle: None,
        }
            .on_change(|v| {
                if v {
                    Message::Enable("adb")
                }else{
                    Message::Disable("adb")
                }
            });

        let ssh = form::Check {
            name: "Enable SSH".to_string(),
            value:cfg.enable_ssh ,
            handle: None,
        }
            .on_change(|v| {
                if v {
                    Message::Enable("ssh")
                }else{
                    Message::Disable("ssh")
                }
            });

        let wifi = Text {
            name: "WiFi".to_string(),
            value: ">".to_string(),
            handle: None,
        }
        .on_select(|| {
            Message::Settings(SettingMessage::NewPage(PageDescription {
                name: "wifi".to_string(),
                state: Default::default(),
            }))
        });
        let file_manager = Text {
            name: "File Manager".to_string(),
            value: ">".to_string(),
            handle: None,
        }
        .on_select(|| {
            Message::Settings(SettingMessage::NewPage(PageDescription {
                name: "file_manager".to_string(),
                state: Default::default(),
            }))
        });
        let shutdown = Text {
            name: "Shutdown".to_string(),
            value: ">".to_string(),
            handle: None,
        }
        .on_select(|| Message::Settings(SettingMessage::ShowShutdownConfirm));
        let reboot = Text {
            name: "Reboot".to_string(),
            value: ">".to_string(),
            handle: None,
        }
        .on_select(|| Message::Settings(SettingMessage::ShowRebootConfirm));
        let inner = Form {
            input: None,
            items: vec![
                volume.into(),
                brightness.into(),
                adb.into(),
                ssh.into(),
                wifi.into(),
                file_manager.into(),
                shutdown.into(),
                reboot.into(),
            ],
            active: 0,
        };
        Ok(Self {
            inner,
            dialog: None,
            pages: vec![],
        })
    }

    pub fn view(&self) -> launcher::Element<'_> {
        if let Some(page) = self.pages.last() {
            return page.view();
        }
        let content = indicator(
            self.inner.view(),
            &[
                (DPad::Up, "Up"),
                (DPad::Down, "Down"),
                (DPad::A, "Select"),
                (DPad::B, "Back"),
            ],
        );
        if let Some(dialog) = &self.dialog {
            iced_widget::stack![content, dialog.view()].into()
        }else{
            content
        }
    }

    pub fn update_networks(&mut self, networks: Vec<WiFi>) {
        for page in self.pages.iter_mut() {
            if let Page::Wifi(page) = page {
                page.update_networks(networks);
                break;
            }
        }
    }

    fn new_page(&mut self, d: PageDescription) {
        if &d.name == "wifi" {
            self.pages.push(Page::Wifi(WifiPage::new()))
        } else if &d.name == "file_manager" {
            self.pages.push(Page::FileManager(FileManager::new("/")))
        }
    }
    pub fn update(&mut self, message: SettingMessage) {
        let s:Vec<String> = (0..20).map(|i|format!("Are you sure? -- {}", i)).collect();
        match message {
            SettingMessage::ShowShutdownConfirm => {
                self.dialog = Some(
                    dialog::Confirm::new("Shutdown Confirm", s.join("\n"))
                        .on_confirm(|b| if b { Message::Shutdown } else { Message::Noop }),
                );
            }
            SettingMessage::ShowRebootConfirm => {
                self.dialog = Some(
                    dialog::Confirm::new("Reboot Confirm", "")
                        .on_confirm(|b| if b { Message::Reboot } else { Message::Noop }),
                );
            }
            SettingMessage::NewPage(page) => {
                self.new_page(page);
            }
        }
    }

    pub fn handle(&mut self, rt: impl LauncherContext, key: DPad) {
        let pop = if let Some(page) = self.pages.last_mut() {
            page.handle(rt, key)
        } else if let Some(dialog) = &mut self.dialog {
            if dialog.handle(rt, key) {
                self.dialog = None;
            }
            return;
        } else {
            self.inner.handle(rt, key);
            return;
        };
        if pop {
            let _ = self.pages.pop();
        }
    }
}
/// 1. CheckBox 设置项

fn volume_to_index(v: u32) -> usize {
    if v >= 100 {
        return 4;
    }
    if v >= 80 {
        return 3;
    }
    if v >= 60 {
        return 2;
    }
    if v >= 40 {
        return 1;
    }
    return 0;
}
fn volume_from_index(v: usize) -> u32 {
    match v {
        0 => 0,
        1 => 40,
        2 => 60,
        3 => 80,
        _ => 100,
    }
}
fn brightness_to_index(v: u32) -> usize {
    if v >= 255 {
        return 4;
    }
    if v >= 150 {
        return 3;
    }
    if v >= 100 {
        return 2;
    }
    if v >= 80 {
        return 1;
    }
    return 0;
}
fn brightness_from_index(v: usize) -> u32 {
    match v {
        0 => 50,
        1 => 80,
        2 => 100,
        3 => 150,
        _ => 255,
    }
}
