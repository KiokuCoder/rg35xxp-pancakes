pub(crate) mod keyboard;
pub(crate) mod list;
pub(crate) mod multi_line_view;
pub(crate) mod dialog;
pub(crate) mod toolkit;
pub(crate) mod form;
pub(crate) mod icon;
pub(crate) mod alert;
pub(crate) mod menu;

pub const MASK_SVG:&[u8] = include_bytes!("../../../assets/mask.svg");
pub const BACKGROUND_SVG:&[u8] = include_bytes!("../../../assets/background.svg");
pub const WALLPAPER: &[u8] = include_bytes!("../../../assets/wallpaper.png");
