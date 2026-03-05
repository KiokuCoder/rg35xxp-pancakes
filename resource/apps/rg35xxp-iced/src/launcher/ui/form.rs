use crate::launcher;
use crate::launcher::executor::Push;
use crate::launcher::ui::keyboard::Keyboard;
use crate::launcher::launcher::Message;
use crate::launcher::ui::list::list_view;
use crate::launcher::pad::DPad;
use crate::launcher::ui;
use crate::launcher::ui::Action;
use crate::launcher::ui::toolkit::{setting_check, setting_switch, setting_text};

pub(crate) enum FormItem<Message, Item: Clone + AsRef<str> = String> {
    Check(Check<Message>),
    Select(Select<Message, Item>),
    Input(Input<Message>),
    Text(Text<Message>),
}
impl<Message, Item: Clone + AsRef<str>> From<Check<Message>> for FormItem<Message, Item> {
    fn from(value: Check<Message>) -> Self {
        FormItem::Check(value)
    }
}
impl<Message, Item: Clone + AsRef<str>> From<Select<Message, Item>>
for FormItem<Message, Item>
{
    fn from(value: Select<Message, Item>) -> Self {
        FormItem::Select(value)
    }
}
impl<Message, Item: Clone + AsRef<str>> From<Input<Message>> for FormItem<Message, Item> {
    fn from(value: Input<Message>) -> Self {
        FormItem::Input(value)
    }
}
impl<Message, Item: Clone + AsRef<str>> From<Text<Message>> for FormItem<Message, Item> {
    fn from(value: Text<Message>) -> Self {
        FormItem::Text(value)
    }
}
pub(crate) struct Text<Message> {
    pub name: String,
    pub value: String,
    pub handle: Option<Box<dyn Fn() -> Message + 'static>>,
}
impl Text<Message> {
    pub fn on_select(self, f: impl Fn() -> Message + 'static) -> Self {
        Self {
            name: self.name,
            value: self.value,
            handle: Some(Box::new(f)),
        }
    }
}
pub(crate) struct Check<Message> {
    pub name: String,
    pub value: bool,
    pub handle: Option<Box<dyn Fn(bool) -> Message + 'static>>,
}
impl Check<Message> {
    pub fn on_change(self, f: impl Fn(bool) -> Message + 'static) -> Self {
        Self {
            name: self.name,
            value: self.value,
            handle: Some(Box::new(f)),
        }
    }
}

pub(crate) struct Select<Message, Item: Clone + AsRef<str>> {
    pub name: String,
    pub value: usize,
    pub options: Vec<Item>,
    pub handle: Option<Box<dyn Fn(Item) -> Message + 'static>>,
}
impl<Message, Item: Clone + AsRef<str>> Select<Message, Item> {
    pub fn on_change(self, f: impl Fn(Item) -> Message + 'static) -> Self {
        Self {
            handle: Some(Box::new(f)),
            ..self
        }
    }
}
pub(crate) struct Input<Message> {
    name: String,
    value: String,
    handle: Option<Box<dyn Fn(String) -> Message + 'static>>,
}
impl<Message> Input<Message> {
    pub fn on_change(self, f: impl Fn(String) -> Message + 'static) -> Self {
        Self {
            handle: Some(Box::new(f)),
            ..self
        }
    }
}

pub(crate) struct Form<Message, Item: Clone + AsRef<str> = String> {
    pub input: Option<Keyboard>,
    pub items: Vec<FormItem<Message, Item>>,
    pub active: usize,
}

impl<Message, Item: Clone + AsRef<str>> Form<Message, Item> {
    pub fn handle(&mut self, rt: impl Push<Message>, key: DPad) {
        if let Some(keyboard) = &mut self.input {
            let ret = keyboard.handle(key);
            match ret {
                None => {return;}
                Some(Action::Submit) => {
                    let txt = keyboard.text.clone();
                    self.input = None;
                    if let Some(FormItem::Input(Input { value, handle, .. })) =
                        self.items.get_mut(self.active)
                    {
                        *value = txt;
                        if let Some(f) = handle {
                            rt.push(f(value.clone()));
                        }
                    }
                    
                }
                Some(Action::Cancel) => {
                    self.input = None;
                    
                }
            }
        }
        match key {
            DPad::Up => {
                self.active = (self.items.len() + self.active - 1) % self.items.len();
            }
            DPad::Down => {
                self.active = (self.active + 1) % self.items.len();
            }
            DPad::Left => {
                if let FormItem::Select(Select {
                                               value,
                                               options,
                                               handle,
                                               ..
                                           }) = &mut self.items[self.active]
                    && *value > 0
                {
                    *value = *value - 1;
                    if let Some(f) = handle
                        && let Some(op) = options.get(*value)
                    {
                        rt.push(f(op.clone()))
                    }
                }
            }
            DPad::Right => {
                if let FormItem::Select(Select {
                                               value,
                                               options,
                                               handle,
                                               ..
                                           }) = &mut self.items[self.active]
                    && *value < options.len() - 1
                {
                    *value += 1;
                    if let Some(f) = handle
                        && let Some(op) = options.get(*value)
                    {
                        rt.push(f(op.clone()))
                    }
                }
            }
            DPad::A => {
                if let FormItem::Check(Check { value, handle, .. }) =
                    &mut self.items[self.active]
                {
                    *value = !*value;
                    if let Some(f) = handle {
                        rt.push(f(*value))
                    }
                }
                if let FormItem::Input(Input { value, .. }) = &self.items[self.active] {
                    self.input = Some(Keyboard::new(value.clone()))
                }
                if let FormItem::Text(Text {
                                             handle: Some(f), ..
                                         }) = &self.items[self.active]
                {
                    rt.push(f());
                }
            }
            DPad::B => {}
            DPad::X => {}
            DPad::Y => {}
            DPad::Select => {}
            DPad::Start => {}
            DPad::Menu => {}
            DPad::L1 => {}
            DPad::R1 => {}
            DPad::L2 => {}
            DPad::R2 => {}
            DPad::Power => {}
            DPad::VolumeUp => {}
            DPad::VolumeDown => {}
        }
    }
    pub fn view(&self) -> launcher::Element<'_> {
        if let Some(input) = &self.input {
            return input.view();
        }
        let mut items: Vec<launcher::Element> = vec![];
        for (idx, x) in self.items.iter().enumerate() {
            let active = self.active == idx;
            match x {
                FormItem::Check(check) => {
                    items.push(setting_check(&check.name, check.value, active));
                }
                FormItem::Select(select) => {
                    items.push(setting_switch(
                        &select.name,
                        select.options[select.value].as_ref(),
                        active,
                    ));
                }
                FormItem::Input(input) => {
                    items.push(setting_text(&input.name, input.value.as_ref(), active));
                }
                FormItem::Text(text) => {
                    items.push(setting_text(&text.name, &text.value, active));
                }
            }
        }
        list_view(items, self.active).into()
    }
}

