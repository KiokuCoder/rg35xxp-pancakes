use crate::launcher;
use crate::launcher::launcher::{LauncherContext, Message};
use crate::launcher::pad::DPad;
use crate::launcher::ui::icon;
use crate::launcher::ui::list::list_view;
use crate::launcher::ui::toolkit::indicator;
use arc_swap::ArcSwap;
use iced_core::{alignment, Background, Border, Color, ContentFit, Length};
use iced_widget::{container, row, stack, text, Column, Container};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::launcher::ui;
use crate::launcher::ui::multi_line_view::multi_line_view;

#[derive(Clone, Debug)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Default,Clone)]
struct State {
    pub loading: bool,
    pub selected: usize,
    pub items: Vec<FileItem>,
    pub root: Option<PathBuf>,
}
pub(crate) struct FileManager {
    pub current_path: PathBuf,
    pub viewing_image: Option<iced::advanced::image::Handle>,
    pub viewing_text: Option<(isize,String)>,
    state: Arc<ArcSwap<State>>,
}

fn welcome_dir()->State{

    State {
        loading: false,
        selected: 0,
        root: None,
        items: vec![
            FileItem {
                name: "Root".to_string(),
                path: PathBuf::from("/"),
                is_dir: true,
            },
            FileItem {
                name: "Home".to_string(),
                path: PathBuf::from("/root"),
                is_dir: true,
            },
            FileItem {
                name: "Sdcard".to_string(),
                path: PathBuf::from("/mnt/mmc"),
                is_dir: true,
            },
        ]
    }

}
impl FileManager {
    pub fn new() -> Self {
        Self {
            current_path: PathBuf::from("/"),
            state: Arc::new(ArcSwap::from_pointee(welcome_dir())),
            viewing_image: None,
            viewing_text: None,
        }
    }

    async fn load_path(state: Arc<ArcSwap<State>>,path: impl AsRef<Path>) {
        let root = state.load().root.clone();
        state.store(Arc::new(State { selected: 0, loading: true, items: vec![], root: root.clone() }));
        let mut items = vec![];
        if let Ok(mut entries) = tokio::fs::read_dir(path.as_ref()).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let is_dir = path.is_dir();
                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                items.push(FileItem { name, path, is_dir });
            }
        }
        // Sort: directories first, then files, then by name
        items.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.cmp(&b.name)
            }
        });
        state.store(Arc::new(State { selected: 0, loading: false, items, root }));
    }

    fn selected(&self, idx: usize) {
        self.set_state(|s| {
            s.selected = idx;
        });
    }

    fn set_state(&self,f:impl Fn(&mut State)) {
        let s = self.state.swap(Arc::new(State { selected: 0, loading: true, items: vec![],root:None }));
        match Arc::try_unwrap(s) {
            Ok(mut s) => {
                f(&mut s);
                self.state.store(Arc::new(s));
            }
            Err(s) => {
                let mut s = (*s).clone();
                f(&mut s);
                self.state.store(Arc::new(s));
            }
        }

    }

    pub fn view(&self) -> launcher::Element<'_> {
        let mut column = Column::new().spacing(5);
        let state:Arc<State> = self.state.load_full();

        if state.loading {
            return indicator(
                container(text("loading...")).center(Length::Fill),
                &[
                    (DPad::Up, "Up"),
                    (DPad::Down, "Down"),
                    (DPad::A, "Open"),
                    (DPad::B, "Back"),
                ],
            ).into();
        }

        // Path header
        let header = container(
            text(format!("Path: {}", self.current_path.display()))
                .size(16)
                .color(Color::from_rgb(0.8, 0.8, 0.8))
        )
        .padding(10)
        .width(Length::Fill)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb8(50, 50, 50))),
            ..Default::default()
        });

        column = column.push(header);

        let mut list_items = Vec::new();
        for (i, item) in state.items.iter().enumerate() {
            let is_selected = i == state.selected;
            list_items.push(self.view_item(item, is_selected));
        }

        let list = list_view(list_items, state.selected);
        
        column = column.push(list);

        let content = container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_|{
                container::Style::default()
                    .background(Background::Color(Color::from_rgba8(0, 0, 0, 0.3)))
            });

        let main_view = indicator(
            content,
            &[
                (DPad::Up, "Up"),
                (DPad::Down, "Down"),
                (DPad::A, "Open"),
                (DPad::B, "Back"),
            ],
        );

        if let Some(image_handle) = &self.viewing_image {
            let overlay = self.view_image_overlay(image_handle.clone());
            stack![main_view, overlay].into()
        } else if let Some((offset, text)) = &self.viewing_text {
            let overlay = container(
                multi_line_view(vec![iced_widget::text(text)], *offset)
            ).style(|_| container::Style::default()
                .background(Background::Color(Color::from_rgb8(250, 250, 250)))
            );
            stack![main_view, overlay].into()
        } else {
            main_view
        }
    }

    fn view_image_overlay<'a>(&self, handle: iced::advanced::image::Handle) -> launcher::Element<'a> {
        let mask = container(
            iced_widget::svg(iced_widget::svg::Handle::from_memory(crate::launcher::ui::MASK_SVG))
                .height(Length::Fill)
                .width(Length::Fill)
                .content_fit(ContentFit::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        let image = iced_widget::image::viewer(handle)
            .width(Length::Fill)
            .height(Length::Fill);

        let image_container = container(image)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        stack![mask, image_container].into()
    }

    fn view_item<'a>(&self, item:&FileItem, is_selected: bool) -> launcher::Element<'a> {
        let icon = if item.is_dir {
            icon::light("folder")
        } else if is_image(&item.path) {
            icon::light("image")
        } else {
            icon::light("files")
        };
        let content = row![
            icon,
            text(item.name.clone()).size(18),
        ]
        .spacing(10)
        .align_y(alignment::Vertical::Center);

        Container::new(content)
            .width(Length::Fill)
            .padding(10)
            .style(move |_| container::Style {
                text_color: Some(Color::WHITE),
                background: if is_selected {
                    Some(Background::Color(Color::from_rgb(0.2, 0.4, 0.8)))
                } else {
                    None
                },
                border: Border {
                    radius: 5.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
            .into()
    }
    pub fn reload(&self, rt: impl LauncherContext) {
        let path = PathBuf::from(&self.current_path);
        let state = self.state.clone();
        rt.spawn(async {
            Self::load_path(state,path).await;
            return Message::Refresh
        });
    }

    pub fn handle(&mut self, rt: impl LauncherContext, key: DPad) -> bool {
        if self.viewing_image.is_some() {
            if matches!(key, DPad::B | DPad::A | DPad::Start | DPad::Select) {
                self.viewing_image = None;
            }
            return false;
        }
        if let Some((offset, _)) = self.viewing_text.as_mut() {
            if key == DPad::Up {
                *offset -= 1;
            } else if key == DPad::Down {
                *offset += 1;
            } else if key == DPad::B {
                self.viewing_text = None;
            }
            return false;
        }
        let state = self.state.load_full();
        let selected = state.selected;

        match key {
            DPad::Up => {
                if selected > 0 {
                    self.selected(selected-1);
                }
            }
            DPad::Down => {
                if !state.items.is_empty() && selected < state.items.len() - 1 {
                    self.selected(selected+1);
                }
            }
            DPad::A => {
                if let Some(item) = state.items.get(selected) {
                    if item.is_dir {
                        self.current_path = item.path.clone();
                        if state.root.is_none() {
                            self.set_state(|s| s.root = Some(item.path.clone()));
                        }
                        self.reload(&rt);
                    } else if is_text(&item.path) {
                        if let Ok(text) = std::fs::read_to_string(&item.path) {
                            self.viewing_text = Some((0, text));
                        }
                    } else if is_image(&item.path) {
                        if let Ok(img) = image::open(&item.path) {
                            let p = img.to_rgba8();
                            self.viewing_image = Some(iced::advanced::image::Handle::from_rgba(
                                p.width(),
                                p.height(),
                                p.into_raw(),
                            ));
                        }
                    }
                }
            }
            DPad::B => {
                if state.root.as_ref().map(|p| p.as_path() == self.current_path.as_path()).unwrap_or_default() {
                    self.state.store(Arc::new(welcome_dir()));
                    return false;
                }
                if let Some(parent) = self.current_path.parent() {
                    if self.current_path != Path::new("/") {
                        self.current_path = parent.to_path_buf();
                        self.reload(&rt);
                    } else {
                        return true;
                    }
                } else {
                    return true; // Exit page
                }
            }
            DPad::X => {
                self.reload(&rt);
            }
            _ =>{}
        }
        false
    }
}

fn is_image(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "bmp" | "gif")
}
fn is_text(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or_default().to_lowercase();
    matches!(ext.as_str(), "txt" | "toml" | "log")
}
