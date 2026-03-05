use crate::launcher;
use crate::launcher::launcher::{LauncherContext, Message};
use crate::launcher::pad::DPad;
use iced_core::{alignment, Background, Border, Color, ContentFit, Length};
use iced_widget::{column, container, space, stack, text};
use crate::launcher::ui::list::list_view;

pub struct Menu {
    pub title: String,
    pub options: Vec<String>,
    pub selected_index: usize,
    pub handle: Option<Box<dyn Fn(usize) -> Message + 'static>>,
    pub cancel: Option<Box<dyn Fn() -> Message + 'static>>,
}

impl Menu {
    pub fn new(title: impl Into<String>, options: Vec<String>) -> Self {
        Self {
            title: title.into(),
            options,
            selected_index: 0,
            handle: None,
            cancel: None,
        }
    }

    pub fn on_select(self, f: impl Fn(usize) -> Message + 'static) -> Self {
        Self {
            handle: Some(Box::new(f)),
            ..self
        }
    }

    pub fn on_cancel(self, f: impl Fn() -> Message + 'static) -> Self {
        Self {
            cancel: Some(Box::new(f)),
            ..self
        }
    }

    pub fn view(&self) -> launcher::Element<'_> {
        let title = text(&self.title)
            .size(24)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        let mut items = Vec::new();
        for (i, option) in self.options.iter().enumerate() {
            items.push(self.menu_item(option, i == self.selected_index));
        }

        let menu_content = container(list_view(items, self.selected_index))
            .max_height(300); // 限制菜单最大高度

        let dialog_box = container(
            column![
                title,
                space().height(10),
                menu_content,
            ]
            .padding(20)
            .width(300),
        )
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb8(40, 40, 40))),
            border: Border {
                color: Color::from_rgb8(100, 100, 100),
                width: 2.0,
                radius: 10.0.into(),
            },
            text_color: Some(Color::WHITE),
            ..Default::default()
        });

        // Overlay
        let mask = container(
            iced_widget::svg(iced_widget::svg::Handle::from_memory(crate::launcher::ui::MASK_SVG))
                .height(Length::Fill)
                .width(Length::Fill)
                .content_fit(ContentFit::Fill)
        )
        .width(Length::Fill)
        .height(Length::Fill);

        let content = container(dialog_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        stack![mask, content].into()
    }

    fn menu_item<'a>(&self, label: &'a str, is_selected: bool) -> launcher::Element<'a> {
        container(text(label).size(20).width(Length::Fill))
            .padding([10, 20])
            .style(move |_| {
                if is_selected {
                    container::Style {
                        background: Some(Background::Color(Color::from_rgb8(45, 95, 200))),
                        text_color: Some(Color::WHITE),
                        ..Default::default()
                    }
                } else {
                    container::Style {
                        text_color: Some(Color::from_rgb8(200, 200, 200)),
                        ..Default::default()
                    }
                }
            })
            .into()
    }

    /// 返回 true 表示菜单关闭
    pub fn handle(&mut self, rt: impl LauncherContext, key: DPad) -> bool {
        match key {
            DPad::Up => {
                if !self.options.is_empty() {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    } else {
                        self.selected_index = self.options.len() - 1;
                    }
                }
                false
            }
            DPad::Down => {
                if !self.options.is_empty() {
                    if self.selected_index < self.options.len() - 1 {
                        self.selected_index += 1;
                    } else {
                        self.selected_index = 0;
                    }
                }
                false
            }
            DPad::A => {
                if let Some(f) = &self.handle {
                    rt.push(f(self.selected_index));
                }
                true
            }
            DPad::B => {
                if let Some(f) = &self.cancel {
                    rt.push(f());
                }
                true
            }
            _ => false,
        }
    }
}
