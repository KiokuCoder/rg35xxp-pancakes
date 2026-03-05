use iced_core::{Color, Length};
use iced_widget::{container, row, text};
use crate::launcher;
use crate::launcher::pad::DPad;
use crate::launcher::launcher::{LauncherContext, Message};
use std::fs;
use std::path::Path;
use anyhow::Context;
use log::info;
use iced_renderer::core::alignment::{Horizontal};
use crate::launcher::ui::toolkit::{indicator, setting_row};

const DEFAULT_ICON: &[u8] = include_bytes!("../../../assets/application.png");
const PAGE_SIZE: usize = 7;

pub(crate) struct Game {
    pub name: String,
    pub path: String,
    pub icon: iced::advanced::image::Handle,
}
pub(crate) struct GameList {
    pub games: Vec<Game>,
    pub selected: usize,
    pub offset: usize,
}

impl GameList {
    pub fn load(p: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root_path = p.as_ref();
        let mut games = Vec::new();

        if !root_path.exists() {
            return Ok(GameList {
                games: Vec::new(),
                selected: 0,
                offset: 0,
            });
        }

        let entries = fs::read_dir(root_path)
            .with_context(|| format!("Failed to read directory: {:?}", root_path))?;

        let img_dir = root_path.join("Imgs");

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("gba") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let icon_path = img_dir.join(format!("{}.png", name));

                let icon_img = if icon_path.exists() {
                    match image::open(&icon_path) {
                        Ok(img) => img,
                        Err(_) => image::load_from_memory(DEFAULT_ICON).context("Failed to load default icon")?
                    }
                } else {
                    image::load_from_memory(DEFAULT_ICON).context("Failed to load default icon")?
                };

                let p = icon_img.to_rgba8();
                games.push(Game {
                    name,
                    path: path.to_string_lossy().to_string(),
                    icon: iced::advanced::image::Handle::from_rgba(p.width(), p.height(), p.into_raw()),
                });
            }
        }

        games.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(GameList {
            games,
            selected: 0,
            offset: 0,
        })
    }

    pub fn view(&self) -> launcher::Element<'_> {
        let total_count = self.games.len();
        if total_count == 0 {
            return indicator(
                container(text("No games found")).width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill),
                &[(DPad::B, "Back")]
            );
        }

        // 计算当前页显示的范围
        let start = self.offset;
        let end = (start + PAGE_SIZE).min(total_count);

        let mut list_items = Vec::new();
        for i in 0..PAGE_SIZE {
            let Some(game) = self.games.get(self.offset + i) else { break; };
            let is_selected = i == self.selected;
            list_items.push(setting_row(&game.name, text(""), is_selected));
        }

        let left_pane = container(iced_widget::column(list_items))
            .width(Length::FillPortion(2))
            .center_x(Length::Fill);

        let right_pane = if let Some(game) = self.games.get(self.offset + self.selected) {
            container(iced_widget::column![
                iced_widget::image::viewer(game.icon.clone())
                    .width(Length::Fixed(240.0))
                    .height(Length::Fixed(160.0)),
                text(&game.name).size(22).width(Length::Fill).align_x(Horizontal::Center),
                text(&game.path).size(14).width(Length::Fill).align_x(Horizontal::Center).color(Color::from_rgb8(180, 180, 180)),
            ].spacing(20).align_x(Horizontal::Center))
            .width(Length::FillPortion(3))
            .height(Length::Fill)
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            container(text("")).width(Length::FillPortion(3)).height(Length::Fill)
        };

        let content = row![left_pane, right_pane]
            .width(Length::Fill)
            .height(Length::Fill);

        indicator(
            content,
            &[
                (DPad::Up, "Up/Down"),
                (DPad::Left, "Prev/Next Page"),
                (DPad::A, "Launch"),
                (DPad::B, "Back"),
            ],
        )
    }

    pub fn handle(&mut self, rt: impl LauncherContext, key: DPad) {
        let total_count = self.games.len();
        if total_count == 0 { return; }

        match key {
            DPad::Up => {
                if self.selected > 0 {
                    // 优先移动光标
                    self.selected -= 1;
                } else if self.offset > 0 {
                    // 光标在顶端且上方有记录，滚动 offset
                    self.offset -= 1;
                } else {
                    // 在最顶端按上，跳转到最末尾
                    self.offset = total_count.saturating_sub(PAGE_SIZE);
                    self.selected = (total_count - self.offset).min(PAGE_SIZE).saturating_sub(1);
                }
            }
            DPad::Down => {
                let items_in_view = (total_count - self.offset).min(PAGE_SIZE);
                if self.selected + 1 < items_in_view {
                    // 优先移动光标
                    self.selected += 1;
                } else if self.offset + PAGE_SIZE < total_count {
                    // 光标在底端且下方有记录，滚动 offset
                    self.offset += 1;
                } else {
                    // 在最底端按下，跳转到最开头
                    self.offset = 0;
                    self.selected = 0;
                }
            }
            DPad::Left => {
                if self.offset == 0 {
                    // 第一页逻辑
                    if self.selected > 0 {
                        // 不在最上面，跳到最上面
                        self.selected = 0;
                    } else {
                        // 在最上面，循环到最后一页最后一条
                        self.offset = total_count.saturating_sub(PAGE_SIZE);
                        let items = (total_count - self.offset).min(PAGE_SIZE);
                        self.selected = items.saturating_sub(1);
                    }
                } else {
                    // 中间页逻辑：只翻页，锁定光标位置
                    self.offset = self.offset.saturating_sub(PAGE_SIZE);
                }
            }
            DPad::Right => {
                // 判断是否已经在最后一页（offset + PAGE_SIZE 覆盖了所有记录）
                if self.offset + PAGE_SIZE >= total_count {
                    // 最后一页逻辑
                    let items_in_view = (total_count - self.offset).min(PAGE_SIZE);
                    if self.selected < items_in_view - 1 {
                        // 不在最下面，跳到最下面
                        self.selected = items_in_view - 1;
                    } else {
                        // 在最下面，循环到第一页第一条
                        self.offset = 0;
                        self.selected = 0;
                    }
                } else {
                    // 中间页逻辑：只翻页，锁定光标位置
                    self.offset += PAGE_SIZE;
                    // 防止 offset 越界
                    if self.offset >= total_count {
                        self.offset = total_count.saturating_sub(PAGE_SIZE);
                    }
                }
            }
            DPad::A | DPad::Start => {
                if let Some(game) = self.games.get(self.offset + self.selected) {
                    info!("Launching game: {}", game.path);
                    let args = vec![
                        "retroarch".to_string(),
                        "-L".to_string(),
                        "/usr/share/libretro/cores/mgba_libretro.so".to_string(),
                        game.path.clone(),
                    ];
                    rt.push(Message::Launch { exec: "rg35xxp-guard".to_string(), wd: "/root".to_string(), args });
                }
            }
            _ => {}
        }

        // 安全性检查：确保 selected 不会因翻页后当前页条目减少而越界
        let items_in_view = (total_count - self.offset).min(PAGE_SIZE);
        if self.selected >= items_in_view {
            self.selected = items_in_view.saturating_sub(1);
        }

        info!("total: {}, offset: {}, selected: {}", total_count, self.offset, self.selected);
    }
}
