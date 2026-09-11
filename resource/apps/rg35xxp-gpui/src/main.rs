use std::borrow::Cow;

use gpui::{
    App, Bounds, Context, FocusHandle, Hsla, KeyBinding, Keystroke, SharedString, Subscription,
    Window, WindowBounds, WindowOptions, actions, div, prelude::*, px, rgb, size,
};
use gpui_platform::application;

actions!(rg35xxp, [SelectPrevious, SelectNext, Confirm, Quit]);

const SWATCHES: [(&str, fn() -> Hsla); 6] = [
    ("Red", gpui::red),
    ("Green", gpui::green),
    ("Blue", gpui::blue),
    ("Yellow", gpui::yellow),
    ("Black", gpui::black),
    ("White", gpui::white),
];

/// The RG35XX Plus driver assigns gamepad codes in an unusual order (L1 is BTN_WEST, Select is
/// BTN_TL, ...), so gpui's Linux-named buttons are translated to what is printed on the device.
fn button_name(key: &str) -> Option<&'static str> {
    Some(match key {
        "btn_south" => "A",
        "btn_east" => "B",
        "btn_c" => "Y",
        "btn_north" => "X",
        "btn_west" => "L1",
        "btn_z" => "R1",
        "btn_tl" => "Select",
        "btn_tr" => "Start",
        "btn_tl2" => "Menu",
        "btn_select" => "L2",
        "btn_start" => "R2",
        "volumeup" => "Vol+",
        "volumedown" => "Vol-",
        "power" => "Power",
        _ => return None,
    })
}

fn describe(keystroke: &Keystroke) -> String {
    match button_name(&keystroke.key) {
        Some(button) => format!("{button} ({keystroke})"),
        None => keystroke.to_string(),
    }
}

struct HelloWorld {
    focus_handle: FocusHandle,
    selected: usize,
    greeting: SharedString,
    /// The last keystroke and how many times in a row it arrived, so held-key repeats show up.
    last_key: Option<(Keystroke, usize)>,
    _keystrokes: Subscription,
}

impl HelloWorld {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        // Interceptors see every key press before bindings consume it; element key listeners
        // only receive keys that no action handled.
        let this = cx.entity().downgrade();
        let keystrokes = cx.intercept_keystrokes(move |event, _, cx| {
            this.update(cx, |this, cx| this.record_keystroke(&event.keystroke, cx))
                .ok();
        });
        Self {
            focus_handle,
            selected: 0,
            greeting: "World".into(),
            last_key: None,
            _keystrokes: keystrokes,
        }
    }

    fn record_keystroke(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
        match &mut self.last_key {
            Some((last, count)) if last == keystroke => *count += 1,
            last_key => *last_key = Some((keystroke.clone(), 1)),
        }
        cx.notify();
    }

    fn select_previous(&mut self, _: &SelectPrevious, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = (self.selected + SWATCHES.len() - 1) % SWATCHES.len();
        cx.notify();
    }

    fn select_next(&mut self, _: &SelectNext, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = (self.selected + 1) % SWATCHES.len();
        cx.notify();
    }

    fn confirm(&mut self, _: &Confirm, _: &mut Window, cx: &mut Context<Self>) {
        if let Some((name, _)) = SWATCHES.get(self.selected) {
            self.greeting = (*name).into();
        }
        cx.notify();
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let key_label = match &self.last_key {
            Some((keystroke, count)) => format!("{} ×{count}", describe(keystroke)),
            None => "Press a button".to_string(),
        };
        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::select_previous))
            .on_action(cx.listener(Self::select_next))
            .on_action(cx.listener(Self::confirm))
            .size_full()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", self.greeting))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .children(SWATCHES.iter().enumerate().map(|(index, (_, color))| {
                        let selected = index == self.selected;
                        div()
                            .size_8()
                            .bg(color())
                            .rounded_md()
                            .when(selected, |swatch| {
                                swatch.border_4().border_color(rgb(0x00a0ff))
                            })
                            .when(!selected, |swatch| {
                                swatch
                                    .border_1()
                                    .border_dashed()
                                    .border_color(rgb(0xa0a0a0))
                            })
                    })),
            )
            .child(div().text_sm().text_color(rgb(0xd0d0d0)).child(key_label))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0xa0a0a0))
                    .child("D-pad: select · A: greet · Menu: quit"),
            )
    }
}

// The device has no IBM Plex Sans, which gpui uses as its default UI font on Linux.
fn load_fonts(cx: &App) {
    let fonts = vec![
        Cow::Borrowed(
            include_bytes!("../../../../zed/assets/fonts/ibm-plex-sans/IBMPlexSans-Regular.ttf")
                .as_slice(),
        ),
        Cow::Borrowed(
            include_bytes!("../../../../zed/assets/fonts/ibm-plex-sans/IBMPlexSans-SemiBold.ttf")
                .as_slice(),
        ),
    ];
    if let Err(error) = cx.text_system().add_fonts(fonts) {
        log::error!("failed to load fonts: {error:#}");
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    application().run(|cx: &mut App| {
        load_fonts(cx);
        cx.bind_keys([
            KeyBinding::new("left", SelectPrevious, None),
            KeyBinding::new("right", SelectNext, None),
            KeyBinding::new("btn_south", Confirm, None),
            KeyBinding::new("enter", Confirm, None),
            KeyBinding::new("btn_tl2", Quit, None),
            KeyBinding::new("escape", Quit, None),
        ]);
        cx.on_action(|_: &Quit, cx| cx.quit());

        let bounds = Bounds::centered(None, size(px(640.), px(480.)), cx);
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| HelloWorld::new(window, cx)),
        );
        if let Err(error) = window {
            log::error!("failed to open window: {error:#}");
            cx.quit();
            return;
        }
        cx.activate(true);
    });
}
