use iced::advanced::layout::{self, Layout, Limits};
use iced::advanced::renderer;
use iced::advanced::widget::{self, Widget};
use iced_core::{
    Background, Color, Element, Length, Point, Rectangle, Size,
};
use iced_core::layout::Node;
use iced_core::mouse::Cursor;
use iced_core::widget::{tree, Tree};

#[derive(Debug, Clone, Copy)]
pub struct MultiLineViewState {
    pub scroll_y: f32,      // 当前真实的像素滚动偏移量
    pub last_offset: isize, // 记录外部传入的滚动触发器上一次的值
}

impl Default for MultiLineViewState {
    fn default() -> Self {
        Self {
            scroll_y: 0.0,
            last_offset: 0,
        }
    }
}

pub struct MultiLineView<'a, Message, Theme, Renderer> {
    pub children: Vec<Element<'a, Message, Theme, Renderer>>,
    pub offset: isize, // 外部传入的滚动触发器（按键按下时增减）
}

impl<'a, Message, Theme, Renderer> MultiLineView<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    pub fn new(children: Vec<Element<'a, Message, Theme, Renderer>>, offset: isize) -> Self {
        Self {
            children,
            offset,
        }
    }
}

impl<'a, Message, Theme, Renderer> From<MultiLineView<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(view: MultiLineView<'a, Message, Theme, Renderer>) -> Self {
        Element::new(view)
    }
}

pub fn multi_line_view<'a, Message, Theme, Renderer>(
    children: Vec<impl Into<Element<'a, Message, Theme, Renderer>>>,
    offset: isize,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    let elements: Vec<Element<'a, Message, Theme, Renderer>> =
        children.into_iter().map(Into::into).collect();

    let view = MultiLineView {
        children: elements,
        offset,
    };
    Element::new(view)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MultiLineView<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        // 1. 确定可视区域大小
        let limits = limits.width(Length::Fill).height(Length::Fill);
        let max_size = limits.max();

        let mut current_y = 0.0;
        let mut nodes = Vec::with_capacity(self.children.len());

        // 2. 构造不受高度限制的子元素 Limits
        // 宽度限制为父容器宽度，高度允许为无穷大
        let child_limits = layout::Limits::new(
            Size::new(max_size.width, 0.0),
            Size::new(max_size.width, f32::INFINITY),
        );

        for (child, child_tree) in self.children.iter_mut().zip(&mut tree.children)
        {
            let mut node = child
                .as_widget_mut()
                .layout(child_tree, renderer, &child_limits);

            node.move_to_mut(Point::new(0.0, current_y));
            current_y += node.bounds().height;
            nodes.push(node);
        }

        let total_height = current_y;
        let view_height = max_size.height;

        let state = tree.state.downcast_mut::<MultiLineViewState>();

        // 3. 处理滚动触发器
        if self.offset != state.last_offset {
            let diff = self.offset - state.last_offset;
            
            let step = 30.0;

            state.scroll_y += diff as f32 * step;
            state.last_offset = self.offset;
        }

        let max_scroll = (total_height - view_height).max(0.0);
        state.scroll_y = state.scroll_y.clamp(0.0, max_scroll);

        for node in &mut nodes {
            let bounds = node.bounds();
            node.move_to_mut(Point::new(bounds.x, bounds.y - state.scroll_y));
        }

        Node::with_children(max_size, nodes)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        if !viewport.intersects(&bounds) {
            return;
        }

        let state = tree.state.downcast_ref::<MultiLineViewState>();
        let list_viewport = viewport.intersection(&bounds).unwrap_or(*viewport);

        renderer.with_layer(bounds, |renderer| {
            for ((child, child_tree), child_layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                let child_bounds = child_layout.bounds();
                if list_viewport.intersects(&child_bounds) {
                    child.as_widget().draw(
                        child_tree,
                        renderer,
                        theme,
                        style,
                        child_layout,
                        cursor,
                        &list_viewport,
                    );
                }
            }
        });

        // 绘制滚动条
        let total_height = layout.children().map(|c| c.bounds().height).sum::<f32>();
        let view_height = bounds.height;

        if total_height > view_height {
            let scrollbar_width = 4.0;
            let scrollbar_margin = 2.0;

            let scrollbar_height = (view_height / total_height) * view_height;
            let scrollbar_y = (state.scroll_y / total_height) * view_height;

            let scrollbar_rect = Rectangle {
                x: bounds.x + bounds.width - scrollbar_width - scrollbar_margin,
                y: bounds.y + scrollbar_y,
                width: scrollbar_width,
                height: scrollbar_height.max(10.0),
            };

            renderer.fill_quad(
                renderer::Quad {
                    bounds: scrollbar_rect,
                    border: iced_core::Border {
                        radius: (scrollbar_width / 2.0).into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                },
                Background::Color(Color::from_rgba8(200, 200, 200, 0.4)),
            );
        }
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<MultiLineViewState>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(MultiLineViewState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }
}
