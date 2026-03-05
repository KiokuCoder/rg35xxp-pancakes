use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum DPad {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    X,
    Y,
    Select,
    Start,
    Menu,
    L1,
    R1,
    L2,
    R2,
    Power,
    VolumeUp,
    VolumeDown,
}
impl AsRef<str> for DPad {
    fn as_ref(&self) -> &str {
        match self {
            DPad::Up => "up",
            DPad::Down => "down",
            DPad::Left => "left",
            DPad::Right => "right",
            DPad::A => "a",
            DPad::B => "b",
            DPad::X => "x",
            DPad::Y => "y",
            DPad::Select => "select",
            DPad::Start => "start",
            DPad::Menu => "menu",
            DPad::L1 => "l1",
            DPad::R1 => "r1",
            DPad::L2 => "l2",
            DPad::R2 => "r2",
            DPad::Power => "power",
            DPad::VolumeUp => "volume-up",
            DPad::VolumeDown => "volume-down",
        }
    }
}
impl Display for DPad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
impl TryFrom<&str> for DPad {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "up" => Ok(DPad::Up),
            "down" => Ok(DPad::Down),
            "left" => Ok(DPad::Left),
            "right" => Ok(DPad::Right),
            "a" => Ok(DPad::A),
            "b" => Ok(DPad::B),
            "x" => Ok(DPad::X),
            "y" => Ok(DPad::Y),
            "select" => Ok(DPad::Select),
            "start" => Ok(DPad::Start),
            "menu" => Ok(DPad::Menu),
            "l1" => Ok(DPad::L1),
            "r1" => Ok(DPad::R1),
            "l2" => Ok(DPad::L2),
            "r2" => Ok(DPad::R2),
            "power" => Ok(DPad::Power),
            "volume-up" => Ok(DPad::VolumeUp),
            "volume-down" => Ok(DPad::VolumeDown),
            &_ => Err("未知映射"),
        }
    }
}
