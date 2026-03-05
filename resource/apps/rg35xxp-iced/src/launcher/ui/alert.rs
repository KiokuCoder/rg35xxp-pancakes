use iced_core::{alignment, Background, Border, Color, ContentFit, Length, Padding};
use iced_widget::{container, space, stack, text};
use crate::launcher;
use crate::launcher::executor::Push;
use crate::launcher::launcher::Message;
use crate::launcher::ui::MASK_SVG;

#[derive(Debug, Clone)]
pub struct Alert {
    pub message: String,
    pub theme: AlertTheme,
}
#[derive(Debug, Clone)]
pub enum AlertTheme {
    Info,
    Error,
    Success,
}

impl Alert {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            theme: AlertTheme::Info,
        }
    }
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            theme: AlertTheme::Error,
        }
    }
    pub fn view(&self) -> launcher::Element<'_> {
        let message = text(&self.message)
            .size(18)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center);

        let dialog_box = container(
            iced_widget::column![space().height(10), message, space().height(20),]
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
            iced_widget::svg(iced_widget::svg::Handle::from_memory(MASK_SVG))
                .height(Length::Fill)
                .width(Length::Fill)
                .content_fit(ContentFit::Fill)
        )
            .padding(Padding::new(0.0).top(0.0))
            .width(Length::Fill)
            .height(Length::Fill);
        let content = container(dialog_box)
            .center_x(Length::Fill)
            .center_y(Length::Fill);
        stack![mask,content].into()
    }
}

pub trait ShowAlert {
    fn alert(&self, _msg: impl Into<String>) {}
}

impl<T> ShowAlert for T
where
    T: Push<Message>,
{
    fn alert(&self, msg: impl Into<String>) {
        self.push(Message::Alert(Alert::info(msg)))
    }
}
