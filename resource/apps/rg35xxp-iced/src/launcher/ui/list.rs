use iced::advanced::layout::Layout;
use iced::advanced::renderer;
use iced::advanced::widget::Widget;
use iced_core::{
    Element, Length, Rectangle, Size, Point,
};
use iced_core::layout::Node;
use iced_core::mouse::Cursor;
use iced_core::widget::{tree, Tree};
#[derive(Debug, Clone, Copy, Default)]
pub struct ListViewState {
    pub offset: f32, // 当前滚动的像素偏移量
}

// ListView 自身只保留外部传入的属性
pub struct ListView<'a, Message, Theme, Renderer> {
    pub children: Vec<Element<'a, Message, Theme, Renderer>>,
    pub selected: usize,
}
impl<'a, Message, Theme, Renderer> From<ListView<'a, Message, Theme, Renderer>>
for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(list_view: ListView<'a, Message, Theme, Renderer>) -> Self {
        // Element::new 会自动将实现了 Widget trait 的类型包装为 Element
        Element::new(list_view)
    }
}

pub fn list_view<'a, Message, Theme, Renderer>(
    children: Vec<impl Into<Element<'a, Message, Theme, Renderer>>>,
    selected: usize,
) -> Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a, // 这里的 renderer::Renderer 需要满足 iced 内部的 Renderer Trait 约束
{
    let elements: Vec<Element<'a, Message, Theme, Renderer>> =
        children.into_iter().map(Into::into).collect();

    let list = ListView {
        children: elements,
        selected,
    };
    Element::new(list)
}

impl<'a, Message, Theme, Renderer> ListView<'a, Message, Theme, Renderer> {
    pub fn new(children: Vec<Element<'a, Message, Theme, Renderer>>, selected: usize) -> Self {
        Self { children, selected }
    }
}
impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
for ListView<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    // ==========================================
    // 核心布局与绘制逻辑
    // ==========================================
    fn size(&self) -> Size<Length> {
        // 假设 ListView 默认填满父容器
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> Node {
        let limits = limits.width(Length::Fill).height(Length::Fill);
        let max_size = limits.max(); // ListView 实际可视窗口大小

        let mut current_y = 0.0;
        let mut nodes = Vec::with_capacity(self.children.len());

        let mut selected_y = 0.0;
        let mut selected_height = 0.0;

        // 1. 遍历并计算所有子元素的初始布局
        for (i, (child, child_tree)) in self.children.iter_mut().zip(&mut tree.children).enumerate()
        {
            // 子元素可以占满宽度，但高度不限 (loose)
            let child_limits = limits.max_width(max_size.width).loose();
            let mut node = child
                .as_widget_mut()
                .layout(child_tree, renderer, &child_limits);

            // 临时设置相对列表顶部的坐标
            node.move_to_mut(Point::new(0.0, current_y));

            // 记录 selected 元素的坐标和高度
            if i == self.selected {
                selected_y = current_y;
                selected_height = node.bounds().height;
            }

            current_y += node.bounds().height;
            nodes.push(node);
        }

        let total_height = current_y;
        let view_height = max_size.height;

        // 获取状态树中的 offset
        let state = tree.state.downcast_mut::<ListViewState>();

        // 2. 自动滚动检测与 offset 调整
        if selected_y < state.offset {
            // selected 元素在可视区域上方（或部分被遮挡），将其与顶部对齐
            state.offset = selected_y;
        } else if selected_y + selected_height > state.offset + view_height {
            // selected 元素在可视区域下方（或部分被遮挡），将其与底部对齐
            state.offset = selected_y + selected_height - view_height;
        }

        // 限制 offset 的范围，防止过度滚动
        let max_offset = (total_height - view_height).max(0.0);
        state.offset = state.offset.clamp(0.0, max_offset);

        // 3. 将 offset 应用于所有子节点
        // 这样做的好处是：在 draw 和 event 处理时，子节点的 Bounds 直接就是屏幕真实坐标
        // 不需要再写复杂的坐标系转换和鼠标偏移逻辑
        for node in &mut nodes {
            let bounds = node.bounds();
            node.move_to_mut(Point::new(bounds.x, bounds.y - state.offset));
        }

        Node::with_children(max_size, nodes)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        // 如果整个 ListView 都不在视口内，直接跳过
        if !viewport.intersects(&bounds) {
            return;
        }

        // 计算当前 ListView 的有效视口裁剪区域
        let list_viewport = viewport.intersection(&bounds).unwrap_or(*viewport);

        // 使用 with_layer 将绘制严格裁剪在 ListView 的 bounds 内
        // 防止列表元素滚动时溢出父容器
        renderer.with_layer(bounds, |renderer| {
            for ((child, child_tree), child_layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                let child_bounds = child_layout.bounds();

                // 视口剔除 (Culling): 只绘制处于可视区域内的子元素
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
    }

    // ==========================================
    // 状态树 (Tree) 必须实现的方法
    // ==========================================
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<ListViewState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(ListViewState::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }
}


