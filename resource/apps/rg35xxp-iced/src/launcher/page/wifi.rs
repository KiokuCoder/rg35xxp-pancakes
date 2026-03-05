use crate::launcher;
use crate::launcher::backend::WiFi;
use crate::launcher::executor::Push;
use crate::launcher::launcher::Message;
use crate::launcher::pad::DPad;
use iced_core::Length;
use iced_widget::{container, text};
use log::info;
use std::collections::HashMap;
use crate::launcher::ui::Action;
use crate::launcher::ui::alert::ShowAlert;
use crate::launcher::ui::keyboard::Keyboard;
use crate::launcher::ui::list::list_view;
use crate::launcher::ui::toolkit::{indicator, setting_row};

pub(crate) struct WifiPage {
    networks: Vec<WiFi>,
    saved: HashMap<String, String>,
    active: usize,
    scanning: bool,
    password: Option<Keyboard>,
    alert: Option<String>,
}

impl WifiPage {
    pub fn new() -> Self {
        Self {
            networks: vec![],
            saved: HashMap::new(),
            active: 0,
            scanning: false,
            password: None,
            alert: None,
        }
    }

    pub fn update_networks(&mut self, networks: Vec<WiFi>) {
        self.networks = networks;
        self.active = 0;
        self.scanning = false;
    }

    pub fn handle(&mut self, rt: impl Push<Message>, key: DPad) -> bool {
        if let Some(keyboard) = &mut self.password {
            if let Some(action) = keyboard.handle(key) {
                if matches!(action, Action::Submit) {
                    if keyboard.text.len() < 8 {
                        rt.alert("Password must be at least 8 bytes.");
                    } else if let Some(network) = self.networks.get(self.active) {
                        info!("Connecting to {}", network.ssid);
                        rt.push(Message::ConnectWifi {
                            ssid: network.ssid.clone(),
                            psk: keyboard.text.clone(),
                        });
                    }
                }
                self.password = None;
            }
            return false;
        }
        match key {
            DPad::Up => {
                if !self.networks.is_empty() {
                    self.active = (self.networks.len() + self.active - 1) % self.networks.len();
                }
            }
            DPad::Down => {
                if !self.networks.is_empty() {
                    self.active = (self.active + 1) % self.networks.len();
                }
            }
            DPad::A => {
                if let Some(network) = self.networks.get(self.active) {
                    if network.security.is_empty() {
                        info!("Connecting to {}", network.ssid);
                        rt.push(Message::ConnectWifi {
                            ssid: network.ssid.clone(),
                            psk: "".to_string(),
                        })
                    } else {
                        self.password = Some(Keyboard::new(String::new()))
                    }
                }
            }
            DPad::B => {
                return true;
            }
            DPad::X => {
                if !self.scanning {
                    self.scanning = true;
                    rt.push(Message::ScanWifi);
                }
            }
            _ => {}
        }
        return false;
    }

    pub fn view(&self) -> launcher::Element<'_> {
        if let Some(keyboard) = &self.password {
            return keyboard.view();
        }
        let mut items: Vec<launcher::Element> = vec![];
        for (idx, network) in self.networks.iter().enumerate() {
            let active = self.active == idx;
            let signal_str = format!("{} dBm", network.signal);
            items.push(setting_row(&network.ssid, text(signal_str), active));
        }

        if items.is_empty() {
            items.push(
                container(text(if self.scanning {
                    "Scanning..."
                } else {
                    "No networks found. Press X to scan."
                }))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into(),
            );
        }

        indicator(
            list_view(items, self.active),
            &[
                (DPad::Up, "Up"),
                (DPad::Down, "Down"),
                (DPad::A, "Connect"),
                (DPad::X, "Scan"),
            ],
        )
    }
}
