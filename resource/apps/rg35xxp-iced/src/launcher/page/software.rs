use crate::launcher;
use crate::launcher::executor::Push;
use crate::launcher::launcher::{LauncherContext, Message};
use crate::launcher::pad::DPad;
use anyhow::Context;
use iced_renderer::core::alignment::{Horizontal, Vertical};
use iced_renderer::core::{Background, Border, Color};
use iced_wgpu::core::Length;
use iced_widget::{container, row, text, Column, Container, stack, svg};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use log::info;
use crate::launcher::ui;
use crate::launcher::ui::toolkit::{cover, indicator};

const DEFAULT_ICON: &[u8] = include_bytes!("../../../assets/application.png");

#[derive(Debug, Deserialize)]
struct SoftwareConfig {
    pub name: String,
    pub cmd: String,
    pub icon: Option<String>,
    pub wd: Option<String>,
}

#[derive(Clone)]
pub(crate) struct Software {
    pub name: String,
    pub icon: iced::advanced::image::Handle,
    pub cmd: String,
    pub wd: String,
}

#[derive(Default)]
pub(crate) struct SoftwareList {
    pub software: Vec<Software>,
    pub offset: usize,
    pub selected_x: usize,
    pub selected_y: usize,
}
// 假设列数固定为 4
const COLUMNS: usize = 4;
// 假设每页行数固定为 3
const ROWS: usize = 3;

impl SoftwareList {
    /// 扫描指定目录下的子文件夹，寻找 info.toml 并加载
    pub fn load(p: impl AsRef<Path>) -> anyhow::Result<Self> {
        let root_path = p.as_ref();
        let mut software_vec = Vec::new();

        // 读取根目录下的内容
        let entries = fs::read_dir(root_path)
            .with_context(|| format!("Failed to read directory: {:?}", root_path))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            info!("Loading software from {:?}", path);

            // 只处理目录
            if path.is_dir() {
                let toml_path = path.join("info.toml");

                // 检查 info.toml 是否存在
                if toml_path.exists() && toml_path.is_file() {
                    // 尝试加载单个软件配置，如果失败打印错误但不中断整个流程
                    match Self::load_single_software(&path, &toml_path) {
                        Ok(software) => software_vec.push(software),
                        Err(e) => {
                            eprintln!("Skipping {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        // 可选：根据名称排序
        software_vec.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(SoftwareList {
            software: software_vec,
            offset: 0,
            selected_x: 0,
            selected_y: 0,
        })
    }

    /// 辅助函数：加载单个软件目录的配置和资源
    fn load_single_software(dir_path: &Path, toml_path: &Path) -> anyhow::Result<Software> {
        // 1. 读取并解析 info.toml
        let content = fs::read_to_string(toml_path).with_context(|| "Failed to read info.toml")?;

        let config: SoftwareConfig =
            toml::from_str(&content).with_context(|| "Failed to parse info.toml structure")?;

        // 2. 加载图标
        // 图标路径是相对于该软件子文件夹的
        let icon_path = dir_path.join(&config.icon.unwrap_or_default());

        let icon_img = if icon_path.exists() {
            image::open(&icon_path)
                .with_context(|| format!("Failed to open icon at {:?}", icon_path))?
        } else {
            // 如果指定的图标不存在，尝试加载默认图标
            // 注意：如果默认图标数据损坏，这里依然会报错
            image::load_from_memory(DEFAULT_ICON)
                .context("Failed to load default icon from memory")?
        };
        let wd = config.wd.unwrap_or(dir_path.to_string_lossy().to_string());
        let p = icon_img.to_rgba8();
        Ok(Software {
            name: config.name,
            icon: iced::advanced::image::Handle::from_rgba(p.width(), p.height(), p.into_raw()),
            cmd: config.cmd,
            wd,
        })
    }
    pub fn view(&self) -> launcher::Element<'_> {
        let mut grid_col = Column::new().spacing(15).padding(10);

        // 遍历视图的每一行 (0..3)
        for view_y in 0..ROWS {
            let mut row_widget = row![].spacing(15);

            // 计算这一行在数据中的真实行号
            let data_row = self.offset + view_y;

            // 遍历视图的每一列 (0..4)
            for view_x in 0..COLUMNS {
                // 计算数据在 Vec 中的一维索引
                // index = (当前行 + 偏移量) * 列宽 + 当前列
                let index = data_row * COLUMNS + view_x;

                // 判断高亮：直接比较相对坐标，非常简单
                let is_selected = view_y == self.selected_y && view_x == self.selected_x;

                // 尝试获取软件数据
                if let Some(soft) = self.software.get(index) {
                    row_widget = row_widget.push(self.view_card(soft, is_selected));
                } else {
                    // 数据越界
                    // 渲染占位符以保持布局对齐
                    row_widget = row_widget.push(self.view_placeholder());
                }
            }
            grid_col = grid_col.push(row_widget);
        }

        indicator(
            container(grid_col)
                .height(Length::Fill)
                .width(Length::Fill)
                .style(|_| container::Style::default()
                    .background(Background::Color(Color::from_rgba8(0, 0, 0, 0.5)))
                ),
            &[
                (DPad::Up, "Move"),
                (DPad::A, "Launch"),
            ],
        )
    }

    // 渲染卡片 (逻辑不变，稍作封装)
    fn view_card<'a>(&'a self, soft: &'a Software, is_selected: bool) -> launcher::Element<'a> {
        let border_color = if is_selected {
            Color::from_rgb(0.2, 0.6, 1.0)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.1)
        };
        
        let background = if is_selected {
            Background::Color(Color::from_rgba(0.2, 0.6, 1.0, 0.2))
        } else {
            Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.4))
        };

        let content = iced_widget::column![
            iced_widget::image::viewer(soft.icon.clone())
                .width(Length::Fixed(56.0))
                .height(Length::Fixed(56.0)),
            text(&soft.name).size(18).width(Length::Fill).align_x(Horizontal::Center)
        ]
        .spacing(8)
        .align_x(Horizontal::Center);

        Container::new(content)
            .width(Length::Fixed(140.0))
            .height(Length::Fixed(100.0))
            .padding(10)
            .style(move |_| container::Style {
                text_color: Some(Color::WHITE),
                background: Some(background),
                border: Border {
                    color: border_color,
                    width: if is_selected { 2.0 } else { 1.0 },
                    radius: 12.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    // 渲染占位符（透明空盒子）
    fn view_placeholder(&self) -> launcher::Element<'_> {
        Container::new(text(""))
            .width(Length::Fixed(140.0))
            .height(Length::Fixed(100.0))
            .into()
    }
    pub fn handle(&mut self,rt:impl LauncherContext, key: DPad) {
        let total_count = self.software.len();
        // 计算总行数
        let total_rows = if total_count == 0 { 0 } else { (total_count + COLUMNS - 1) / COLUMNS };

        match key {
            // --- 向上 ---
            DPad::Up => {
                if self.selected_y > 0 {
                    // 还在视图中间，直接视觉上移
                    self.selected_y -= 1;
                } else if self.offset > 0 {
                    // 在视图顶部，向上滚动数据
                    self.offset -= 1;
                }
            }

            // --- 向下 ---
            DPad::Down => {
                let current_abs_row = self.offset + self.selected_y;

                // 只有当“下一行”存在时才移动
                if current_abs_row + 1 < total_rows {
                    // 1. 移动 Y 轴或 Offset
                    if self.selected_y < ROWS - 1 {
                        self.selected_y += 1;
                    } else {
                        self.offset += 1;
                    }

                    // 2. 检查 X 轴是否越界（吸附逻辑）
                    let new_abs_row = self.offset + self.selected_y;
                    let start_idx = new_abs_row * COLUMNS;
                    let items_in_row = if start_idx + COLUMNS <= total_count {
                        COLUMNS
                    } else {
                        total_count - start_idx
                    };

                    if self.selected_x >= items_in_row {
                        self.selected_x = items_in_row.saturating_sub(1);
                    }
                }
            }

            // --- 向左 ---
            DPad::Left => {
                if self.selected_x > 0 {
                    self.selected_x -= 1;
                }
            }

            // --- 向右 ---
            DPad::Right => {
                let current_abs_row = self.offset + self.selected_y;
                let current_idx = current_abs_row * COLUMNS + self.selected_x;

                if self.selected_x < COLUMNS - 1 && current_idx + 1 < total_count {
                    self.selected_x += 1;
                }
            }

            // --- 确认/启动 ---
            DPad::A | DPad::Start => {
                let idx = (self.offset + self.selected_y) * COLUMNS + self.selected_x;
                if let Some(soft) = self.software.get(idx) {
                    rt.push(Message::Launch {
                        exec: "bash".to_string(),
                        wd: soft.wd.clone(),
                        args: vec!["-c".to_string(), soft.cmd.clone()],
                    });
                }
            }
            DPad::Menu => {
                rt.push(Message::Screenshot);
            }

            // 其他按键不做处理
            _ => {}
        }
    }
}
