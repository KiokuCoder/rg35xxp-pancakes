pub(crate) mod pad;
pub(crate) mod backend;
pub(crate) mod launcher;
pub(crate) mod executor;
pub(crate) mod page;
pub(crate) mod config;
pub(crate) mod fonts;
pub(crate) mod ui;

#[cfg(feature = "rg35xxp")]
type Element<'a> = iced_core::Element<'a, launcher::Message, iced_core::Theme, iced_wgpu::Renderer>;
trait Widget: iced_core::Widget<launcher::Message, iced_core::Theme, iced_wgpu::Renderer> {}

#[cfg(not(feature = "rg35xxp"))]
type Element<'a> = iced::Element<'a, launcher::Message>;
