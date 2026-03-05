use crate::launcher;
use crate::launcher::pad::DPad;
use iced_core::alignment::Horizontal;
use iced_core::{Alignment, Color, Length};
use iced_widget::{container, row, text, Column};
use crate::launcher::ui::toolkit::indicator;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardMode {
    Lowercase,
    Uppercase,
    Symbols,
}

pub struct Keyboard {
    pub current_row: usize,
    pub current_col: usize,
    pub mode: KeyboardMode,
    pub text: String,
}

impl Keyboard {
    pub fn new(text: String) -> Self {
        Self {
            current_row: 0,
            current_col: 0,
            mode: KeyboardMode::Lowercase,
            text,
        }
    }

    fn layouts(&self) -> Vec<Vec<&'static str>> {
        match self.mode {
            KeyboardMode::Lowercase => vec![
                vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
                vec!["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
                vec!["a", "s", "d", "f", "g", "h", "j", "k", "l"],
                vec!["z", "x", "c", "v", "b", "n", "m"],
            ],
            KeyboardMode::Uppercase => vec![
                vec!["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
                vec!["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
                vec!["A", "S", "D", "F", "G", "H", "J", "K", "L"],
                vec!["Z", "X", "C", "V", "B", "N", "M"],
            ],
            KeyboardMode::Symbols => vec![
                vec!["!", "@", "#", "$", "%", "^", "&", "*", "(", ")"],
                vec!["~", "`", "-", "_", "=", "+", "[", "]", "{", "}"],
                vec!["\\", "|", ";", ":", "'", "\"", "?", "/"],
                vec!["<", ">", " ", ",", "."],
            ],
        }
    }

    pub fn handle(&mut self, key: DPad) -> bool {
        let layout = self.layouts();
        match key {
            DPad::Up => {
                if self.current_row > 0 {
                    self.current_row -= 1;
                } else {
                    self.current_row = layout.len() - 1;
                }
                if self.current_col >= self.layouts()[self.current_row].len() {
                    self.current_col = self.layouts()[self.current_row].len() - 1;
                }
            }
            DPad::Down => {
                if self.current_row < layout.len() - 1 {
                    self.current_row += 1;
                } else {
                    self.current_row = 0;
                }
                if self.current_col >= self.layouts()[self.current_row].len() {
                    self.current_col = self.layouts()[self.current_row].len() - 1;
                }
            }
            DPad::Left => {
                if self.current_col > 0 {
                    self.current_col -= 1;
                } else {
                    self.current_col = layout[self.current_row].len() - 1;
                }
            }
            DPad::Right => {
                if self.current_col < layout[self.current_row].len() - 1 {
                    self.current_col += 1;
                } else {
                    self.current_col = 0;
                }
            }
            DPad::A => {
                let char = layout[self.current_row][self.current_col];
                self.text.push_str(char);
            }
            DPad::X => {
                self.text.pop();
            }
            DPad::Y => {
                self.mode = match self.mode {
                    KeyboardMode::Lowercase => KeyboardMode::Uppercase,
                    KeyboardMode::Uppercase => KeyboardMode::Symbols,
                    KeyboardMode::Symbols => KeyboardMode::Lowercase,
                };
            }
            DPad::Start => {
                return true; // 结束输入
            }
            _ => {}
        }

        let layout = self.layouts();
        if self.current_col >= layout[self.current_row].len() {
            self.current_col = layout[self.current_row].len() - 1;
        }
        return false;
    }

    pub fn view(&self) -> launcher::Element<'_> {
        let layout = self.layouts();

        let mut column = Column::new().spacing(8).align_x(Alignment::Center);

        // 输入框显示区
        column = column.push(
            container(text(&self.text).size(28))
                .padding(12)
                .style(|_| container::Style {
                    background: Some(iced_core::Background::Color(Color::from_rgb8(40, 40, 40))),
                    border: iced_core::border::Border {
                        color: Color::from_rgb8(100, 100, 100),
                        width: 2.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .width(Length::Fill)
                .height(60)
                .center_y(Length::Fill),
        );

        // 键盘网格
        for (r, row_keys) in layout.iter().enumerate() {
            let mut row_widget = row![].spacing(5).width(Length::Shrink);
            for (c, key_text) in row_keys.iter().enumerate() {
                let is_selected = r == self.current_row && c == self.current_col;
                let key_display = if *key_text == " " { "␣" } else { *key_text };

                let btn = container(text(key_display).size(24))
                    .width(50)
                    .height(40)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .center_y(Length::Fill)
                    .style(move |_| container::Style {
                        background: if is_selected {
                            Some(iced_core::Background::Color(Color::from_rgba8(
                                180, 0, 0, 0.5,
                            )))
                        } else {
                            Some(iced_core::Background::Color(Color::from_rgba8(
                                100, 100, 100, 0.5,
                            )))
                        },
                        border: iced_core::border::Border {
                            color: if is_selected {
                                Color::WHITE
                            } else {
                                Color::TRANSPARENT
                            },
                            width: if is_selected { 2.0 } else { 0.0 },
                            radius: 6.0.into(),
                        },
                        ..Default::default()
                    });

                row_widget = row_widget.push(btn);
            }
            column = column.push(container(row_widget).align_x(Horizontal::Center));
        }

        // 底部状态与操作提示
        let _mode_name = match self.mode {
            KeyboardMode::Lowercase => "abc",
            KeyboardMode::Uppercase => "ABC",
            KeyboardMode::Symbols => "#+=",
        };

        indicator(container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(15),&[
            (DPad::Y,"Mode"),
            (DPad::X,"Delete"),
            (DPad::A,"Select"),
            (DPad::B,"Back"),
        ])
    }
}
