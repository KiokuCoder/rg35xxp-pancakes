use iced_renderer::core::keyboard::key::Named;
use std::collections::HashMap;

pub(crate) fn k_map() -> HashMap<String, crate::launcher::pad::DPad> {
    HashMap::from([
        (format!("{:?}", Named::ArrowUp), crate::launcher::pad::DPad::Up),
        (
            format!("{:?}", Named::ArrowDown),
            crate::launcher::pad::DPad::Down,
        ),
        (
            format!("{:?}", Named::ArrowLeft),
            crate::launcher::pad::DPad::Left,
        ),
        (
            format!("{:?}", Named::ArrowRight),
            crate::launcher::pad::DPad::Right,
        ),
        (format!("{:?}", Named::Enter), crate::launcher::pad::DPad::A),
        (format!("{:?}", Named::Backspace), crate::launcher::pad::DPad::B),
        (format!("{:?}", Named::Space), crate::launcher::pad::DPad::Start),
        (format!("{:?}", Named::PageUp), crate::launcher::pad::DPad::L1),
        (format!("{:?}", Named::PageDown), crate::launcher::pad::DPad::R1),
        (format!("{:?}", Named::Home), crate::launcher::pad::DPad::L2),
        (format!("{:?}", Named::End), crate::launcher::pad::DPad::R2),
        (format!("{:?}", Named::Tab), crate::launcher::pad::DPad::Menu),
        (format!("{:?}", Named::Escape), crate::launcher::pad::DPad::Power),
        ("-".to_string(), crate::launcher::pad::DPad::VolumeUp),
        ("=".to_string(), crate::launcher::pad::DPad::VolumeDown),
    ])
}
pub fn run() -> anyhow::Result<()> {
    todo!()
}
