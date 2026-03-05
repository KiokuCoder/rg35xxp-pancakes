mod egl;

use calloop::EventLoop;
use egl::eglGetProcAddress;
use egui::{Key, Modifiers};
use egui_glow::glow;
use input_linux::{AbsoluteAxis, Event};
use std::fs::File;
use std::os::fd::AsRawFd;
use std::time::Instant;

#[derive(Default)]
pub struct Builder<T: 'static + SharedState + Default> {
    _unused: T,
    event_loop_init: Vec<Box<dyn FnOnce(&EventLoop<T>)>>,
}

impl<'l, T> Builder<T>
where
    T: 'static + SharedState + Default,
{
    pub fn new() -> Self {
        Builder {
            _unused: T::default(),
            event_loop_init: Vec::new(),
        }
    }
    pub fn event(&mut self, callback: impl FnOnce(&EventLoop<T>) + 'static) {
        self.event_loop_init.push(Box::new(callback));
    }

    pub fn run(self, mut shared_data: T, mut drawer: impl FnMut(&mut T, &mut egui::Ui)) {
        let window = unsafe { egl::FramebufferWindow::new() };
        let (w, h) = window.size();
        println!("OpenGL ES 2.0 context created");

        let glow_context = unsafe {
            glow::Context::from_loader_function(|s| {
                let cstr = std::ffi::CString::new(s).unwrap();
                eglGetProcAddress(cstr.as_ptr()) as *const _
            })
        }
        .into();
        let egui_ctx = egui::Context::default();
        let mut painter = egui_glow::Painter::new(glow_context, "", None, false).unwrap();

        let start_time = Instant::now();

        let mut event_loop: EventLoop<T> =
            EventLoop::try_new().expect("Failed to initialize the event loop!");

        let handle = event_loop.handle();
        let (sender, rec) = calloop::channel::channel::<input_linux::Event>();
        let dev = File::open("/dev/input/event1").unwrap();
        std::thread::spawn(move || {
            let input_handle = unsafe { input_linux::evdev::EvdevHandle::from_fd(dev.as_raw_fd()) };
            while let Ok(event) = input_handle.read_event() {
                sender.send(event).unwrap();
            }
        });

        handle
            .insert_source(rec, move |event_wrapper, meta, shared_data| {
                match event_wrapper {
                    calloop::channel::Event::Msg(event) => {
                        match event {
                            Event::Synchronize(_) => {}
                            Event::Key(key) => {
                                let code = key.key.code();
                                if key.value.is_pressed() {
                                    println!("Key pressed: {}", code);
                                } else {
                                    println!("Key released: {}", code);
                                }
                                if code == 304 && key.value.is_pressed() {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Enter,
                                        physical_key: None,
                                        pressed: true,
                                        repeat: false,
                                        modifiers: Modifiers::default(),
                                    });
                                } else if code == 304 {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Enter,
                                        physical_key: None,
                                        pressed: false,
                                        repeat: false,
                                        modifiers: Modifiers::default(),
                                    });
                                } else if code == 305 && key.value.is_pressed() {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Escape,
                                        physical_key: None,
                                        pressed: true,
                                        repeat: false,
                                        modifiers: Modifiers::default(),
                                    });
                                } else if code == 305 {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Escape,
                                        physical_key: None,
                                        pressed: false,
                                        repeat: false,
                                        modifiers: Modifiers::default(),
                                    });
                                } else if code == 308 && key.value.is_pressed() {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Tab,
                                        physical_key: None,
                                        pressed: true,
                                        repeat: false,
                                        modifiers: Modifiers::SHIFT,
                                    });
                                } else if code == 308 {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Tab,
                                        physical_key: None,
                                        pressed: false,
                                        repeat: false,
                                        modifiers: Default::default(),
                                    });
                                } else if code == 309 && key.value.is_pressed() {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Tab,
                                        physical_key: None,
                                        pressed: true,
                                        repeat: false,
                                        modifiers: Modifiers::default(),
                                    });
                                } else if code == 309 {
                                    shared_data.input().events.push(egui::Event::Key {
                                        key: Key::Tab,
                                        physical_key: None,
                                        pressed: false,
                                        repeat: false,
                                        modifiers: Default::default(),
                                    });
                                }
                            }
                            Event::Relative(_) => {}
                            Event::Absolute(abs) => {
                                // 这里需要把手柄的输入映射转换成为键盘上下左右
                                if abs.axis == AbsoluteAxis::Hat0Y {
                                    // 上下按键
                                    if abs.value > 0 {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowDown,
                                            physical_key: None,
                                            pressed: true,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        shared_data.hat().down = true;
                                    } else if abs.value < 0 {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowUp,
                                            physical_key: None,
                                            pressed: true,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        // shared_data.hat().up = true;
                                    } else if shared_data.hat().down {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowDown,
                                            physical_key: None,
                                            pressed: false,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        // shared_data.down = false;
                                    } else if shared_data.hat().up {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowUp,
                                            physical_key: None,
                                            pressed: false,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        shared_data.hat().up = false;
                                        println!("Up released");
                                    }
                                }
                                if abs.axis == AbsoluteAxis::Hat0X {
                                    // 左右按键
                                    if abs.value > 0 {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowRight,
                                            physical_key: None,
                                            pressed: true,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        shared_data.hat().right = true;
                                    } else if abs.value < 0 {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowLeft,
                                            physical_key: None,
                                            pressed: true,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        shared_data.hat().left = true;
                                    } else if shared_data.hat().right {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowRight,
                                            physical_key: None,
                                            pressed: false,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        shared_data.hat().right = false;
                                    } else if shared_data.hat().left {
                                        shared_data.input().events.push(egui::Event::Key {
                                            key: Key::ArrowLeft,
                                            physical_key: None,
                                            pressed: false,
                                            repeat: false,
                                            modifiers: Default::default(),
                                        });
                                        // shared_data.left = false;
                                    }
                                }
                            }
                            Event::Switch(_) => {}
                            Event::Misc(_) => {}
                            Event::Led(_) => {}
                            Event::Autorepeat(_) => {}
                            Event::Sound(_) => {}
                            Event::ForceFeedback(_) => {}
                            Event::ForceFeedbackStatus(_) => {}
                            Event::UInput(_) => {}
                            Event::Unknown(e) => {}
                        }
                    }
                    calloop::channel::Event::Closed => {}
                }
            })
            .expect("TODO: panic message");
        for x in self.event_loop_init {
            x(&mut event_loop)
        }

        egui_ctx.set_pixels_per_point(2.0); // 缩放比例，200%缩放
        event_loop
            .run(
                std::time::Duration::from_secs_f32(1.0) / 60,
                &mut shared_data,
                |shared_data| {
                    shared_data.input().time = Some(start_time.elapsed().as_secs_f64());
                    egui_ctx.begin_pass(shared_data.input().take());

                    egui::CentralPanel::default().show(&egui_ctx, |ui| drawer(shared_data, ui));

                    let egui::FullOutput {
                        platform_output,
                        textures_delta,
                        shapes,
                        pixels_per_point,
                        viewport_output,
                    } = egui_ctx.end_pass();

                    for (id, image_delta) in textures_delta.set {
                        painter.set_texture(id, &image_delta);
                    }

                    let clipped_primitives = egui_ctx.tessellate(shapes, pixels_per_point);
                    painter.paint_primitives([w, h], pixels_per_point, &clipped_primitives);

                    for id in textures_delta.free {
                        painter.free_texture(id);
                    }

                    window.present()
                },
            )
            .expect("Error during event loop!");
    }
}

#[derive(Default)]
pub struct HatState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}
pub trait SharedState {
    fn input(&mut self) -> &mut egui::RawInput;
    fn hat(&mut self) -> &mut HatState;
}
