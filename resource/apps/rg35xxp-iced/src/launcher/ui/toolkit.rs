use std::ops::RangeInclusive;
use iced_core::{alignment, Background, Color, ContentFit, Length};
use iced_core::text::Wrapping;
use iced_widget::{checkbox, container, row, slider, text};
use crate::launcher::Element;
use crate::launcher::launcher::Message;
use crate::launcher::pad::DPad;
use crate::launcher::ui::BACKGROUND_SVG;

pub fn setting_check(label: &str, is_checked: bool, is_active: bool) -> Element<'_> {
    setting_row(label, checkbox(is_checked), is_active)
}

/// 2. Switch (左右切换) 设置项
pub fn setting_switch<'a>(label: &'a str, current_option: &'a str, is_active: bool) -> Element<'a> {
    let selector = row![
        text("< ").size(16).color(if is_active {
            Color::WHITE
        } else {
            Color::from_rgb8(140, 140, 140)
        }),
        container(text(current_option).size(14))
            .padding([2, 12])
            .style(move |_| container::Style {
                background: Some(Background::Color(if is_active {
                    Color::from_rgb8(60, 110, 220)
                } else {
                    Color::from_rgba8(50, 50, 50, 0.7)
                })),
                border: iced_core::border::Border {
                    color: if is_active {
                        Color::from_rgb8(100, 150, 255)
                    } else {
                        Color::from_rgba8(100, 100, 100, 0.5)
                    },
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            }),
        text(" >").size(16).color(if is_active {
            Color::WHITE
        } else {
            Color::from_rgb8(140, 140, 140)
        }),
    ]
        .align_y(alignment::Vertical::Center);

    setting_row(label, selector, is_active)
}

/// 3. Collapse (折叠/展开) 设置项
pub fn setting_collapse(
    label: &str,
    is_active: bool,
    // is_expanded: bool, // 后续可根据这个参数决定显示 ▹ 还是 ▿
) -> Element<'_> {
    setting_row(
        label,
        text(">").size(18).color(Color::from_rgb8(150, 150, 150)),
        is_active,
    )
}

/// 4. Slider 滑动条设置项
pub fn setting_slide<'a>(
    label: &'a str,
    range: RangeInclusive<f32>,
    value: f32,
    on_change: impl Fn(f32) -> Message + 'a,
    is_active: bool,
) -> Element<'a> {
    setting_row(
        label,
        slider(range, value, on_change).width(Length::Fixed(120.0)),
        is_active,
    )
}

/// 5. 纯文本展示设置项
pub fn setting_text<'a>(label: &'a str, value: &'a str, is_active: bool) -> Element<'a> {
    let val_display = container(text(value).size(14))
        .padding([4, 12])
        .style(move |_| container::Style {
            background: Some(Background::Color(Color::from_rgba8(255, 255, 255, 0.05))),
            border: iced_core::border::Border {
                color: Color::from_rgba8(255, 255, 255, 0.1),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });
    setting_row(label, val_display, is_active)
}

/// 辅助组件：用于生成统一格式的“左文本-右控件”行
pub(crate) fn setting_row<'a>(
    label: &'a str,
    right_widget: impl Into<Element<'a>>,
    is_active: bool, // 新增参数：当前行是否处于激活/选中状态
) -> Element<'a> {
    // 基础行布局
    let content = row![
        text(label).size(16).width(Length::Fill).wrapping(Wrapping::None),
        right_widget.into()
    ]
        .width(Length::Fill)
        .align_y(alignment::Vertical::Center);

    // 用 Container 包裹，以便控制内边距和背景色
    let item_container = container(content)
        .padding([12, 16]) // 把内边距提上来，让高亮背景能够填满整个区块
        .style(move |_theme| {
            if is_active {
                // Active 状态下的样式：具有质感的蓝色，带微妙的边框
                container::Style {
                    background: Some(Background::Color(Color::from_rgba8(45, 95, 200, 0.9))),
                    text_color: Some(Color::WHITE),
                    border: iced_core::border::Border {
                        color: Color::from_rgb8(100, 150, 255),
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            } else {
                // 普通状态：半透明深灰色背景，增加柔和的边框
                container::Style {
                    background: Some(Background::Color(Color::from_rgba8(30, 30, 30, 0.6))),
                    text_color: Some(Color::from_rgb8(200, 200, 200)),
                    border: iced_core::border::Border {
                        color: Color::from_rgba8(255, 255, 255, 0.1),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                }
            }
        });

    container(item_container)
        .padding([4, 8]) // 增加上下左右的间距，提升质感
        .into()
}
pub fn indicator<'a>(
    fragment: impl Into<Element<'a>>,
    buttons: &'static [(DPad, &'static str)],
) -> Element<'a> {
    let content = container(fragment)
        .height(Length::Fill)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.4))),
            ..Default::default()
        });

    let mut indicator_row = row![].spacing(16).align_y(alignment::Vertical::Center);

    for (key, action) in buttons {
        let key_label = match key {
            DPad::Up => "UP",
            DPad::Down => "DOWN",
            DPad::Left => "LEFT",
            DPad::Right => "RIGHT",
            DPad::A => "A",
            DPad::B => "B",
            DPad::X => "X",
            DPad::Y => "Y",
            DPad::Select => "SEL",
            DPad::Start => "STA",
            DPad::Menu => "MENU",
            _ => key.as_ref(),
        };

        // 按键“质感”：深色背景 + 细边框 + 圆角
        let key_cap = container(text(key_label).size(18).color(Color::WHITE))
            .padding([2, 8])
            .style(|_| container::Style {
                background: Some(Background::Color(Color::from_rgb8(60, 60, 60))),
                border: iced_core::border::Border {
                    color: Color::from_rgb8(90, 90, 90),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            });

        // 提示文字
        let action_label = text(*action)
            .size(22)
            .color(Color::from_rgb8(210, 210, 210));

        indicator_row = indicator_row.push(
            row![key_cap, action_label]
                .spacing(8)
                .align_y(alignment::Vertical::Center),
        );
    }

    let footer = container(
        row![
            iced_widget::space(),
            indicator_row,
            iced_widget::space().height(20)
        ]
            .height(Length::Fill)
            .align_y(alignment::Vertical::Center),
    )
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgba8(20, 20, 20, 0.85))),
            // 顶部边框线，增加层次感
            border: iced_core::border::Border {
                color: Color::from_rgba8(60, 60, 60, 0.4),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..Default::default()
        })
        .height(44)
        .width(Length::Fill);

    iced_widget::column![content, footer].into()
}

pub fn cover<'a>(content: impl Into<Element<'a>>) -> iced_widget::Stack<'a, Message, iced_core::Theme, iced_wgpu::Renderer> {
    let background: Element<'a> = container(
        iced_widget::svg(iced_widget::svg::Handle::from_memory(BACKGROUND_SVG))
            .height(Length::Fill)
            .width(Length::Fill)
            .content_fit(ContentFit::Fill)
    )
        .center_x(Length::Fill)
        .center_y(Length::Fill).into();
    let content: Element<'a> = container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill).into();
    iced_widget::stack([background, content])
}
