use crate::launcher;
use crate::launcher::launcher::{LauncherContext, Message};
use crate::launcher::pad::DPad;
use iced_core::{alignment, Background, Border, Color, ContentFit, Length, Padding};
use iced_widget::{column, container, row, space, stack, text};
use crate::launcher::ui::multi_line_view::multi_line_view;

pub struct Confirm {
    pub title: String,
    pub message: String,
    pub message_offset: isize, // 提醒消息滚动
    pub handle: Option<Box<dyn Fn(bool) -> Message + 'static>>,
    pub selected_index: usize, // 0 for OK/Confirm, 1 for Cancel (if Confirm)
}

impl Confirm {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            message_offset: 0,
            handle: None,
            selected_index: 0,
        }
    }
    pub fn on_confirm(self, f: impl Fn(bool) -> Message + 'static) -> Self {
        Self {
            handle: Some(Box::new(f)),
            ..self
        }
    }

    pub fn view(&self) -> launcher::Element<'_> {
        let title = text(&self.title)
            .size(24)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        let message = text(&self.message)
            .size(18)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        let message = container(multi_line_view(vec![message], self.message_offset))
            .height(120);

        let buttons = row![
            self.button("Confirm", self.selected_index == 0),
            self.button("Cancel", self.selected_index == 1),
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center);

        let dialog_box = container(
            column![
                title,
                space().height(10),
                message,
                space().height(20),
                container(buttons)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Center),
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
        let mask  = container(
            iced_widget::svg(iced_widget::svg::Handle::from_memory(crate::launcher::ui::MASK_SVG))
                .height(Length::Fill)
                .width(Length::Fill)
                .content_fit(ContentFit::Fill)
        )
            .padding(Padding::new(0.0).top(0.0))
            .width(Length::Fill)
            .height(Length::Fill);
        let content = container(dialog_box)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        stack![mask,content].into()
    }

    fn button<'a>(&self, label: &'a str, is_selected: bool) -> launcher::Element<'a> {
        container(text(label).size(18))
            .padding([8, 20])
            .style(move |_| {
                if is_selected {
                    container::Style {
                        background: Some(Background::Color(Color::from_rgb8(45, 95, 200))),
                        border: Border {
                            color: Color::from_rgb8(80, 130, 255),
                            width: 2.0,
                            radius: 5.0.into(),
                        },
                        text_color: Some(Color::WHITE),
                        ..Default::default()
                    }
                } else {
                    container::Style {
                        background: Some(Background::Color(Color::from_rgb8(60, 60, 60))),
                        border: Border {
                            color: Color::from_rgb8(80, 80, 80),
                            width: 1.0,
                            radius: 5.0.into(),
                        },
                        text_color: Some(Color::from_rgb8(200, 200, 200)),
                        ..Default::default()
                    }
                }
            })
            .into()
    }

    /// 如果返回 true 表示关闭 Dialog
    pub fn handle(&mut self, rt: impl LauncherContext, key: DPad) -> bool {
        match key {
            DPad::Up  => {
                self.message_offset -= 1;
                false
            }
            DPad::Down  => {
                self.message_offset += 1;
                false
            }
            DPad::Left | DPad::Right => {
                self.selected_index = (self.selected_index + 1) % 2;
                false
            }
            DPad::A => {
                if let Some(f) = &self.handle {
                    rt.push(f(self.selected_index == 0))
                }
                true
            }
            DPad::B => {
                if let Some(f) = &self.handle {
                    rt.push(f(false))
                }
                true
            }
            _ => false,
        }
    }
}
