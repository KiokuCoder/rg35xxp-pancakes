use log::{error};
use std::process::Command;
use anyhow::anyhow;
use iced_core::mouse::Cursor;
use iced_core::{clipboard, renderer, Event, Size, Theme};
use iced_runtime::{user_interface, UserInterface};
use rodio::MixerDeviceSink;
use crate::rg35xxp::backlight::RG35xxpBackend;
use crate::rg35xxp::egl::FramebufferWindow;
use crate::rg35xxp::input;
use crate::rg35xxp::render::Wrap;
use crate::launcher::executor;
use crate::launcher::executor::Pull;
use crate::launcher::launcher::{Launcher, Message};
use crate::launcher::pad::DPad;

fn play_wav(mixer:&MixerDeviceSink, data:&'static [u8])  {
    match rodio::Decoder::new_wav(std::io::Cursor::new(data)) {
        Ok(source) => {
            mixer.mixer().add(source);
        }
        Err(err) => {
            error!("解码 wav 异常: {}", err)
        }
    }
}

#[cfg(feature = "rg35xxp")]
pub fn run(state: &mut Launcher<RG35xxpBackend>) -> anyhow::Result<Next> {
    // 使用 alsa 驱动会独占声卡，所以需要确保退出的时候解除声卡占用
    let mixer = rodio::DeviceSinkBuilder::open_default_sink()?;
    let fb = unsafe { FramebufferWindow::new() }.map_err(|err| anyhow!("{}", err))?;
    if let Err(e) = fb.set_swap_interval(1) {
        return Err(anyhow!("Warning: VSync setup failed: {}", e));
    };
    let mut wrap = Wrap::from(fb).map_err(|err| anyhow!("{}", err))?;
    let mut cache = user_interface::Cache::new();
    let mut messages: Vec<Message> = vec![];
    let (push, mut pull) = executor::new::<Message>();
    let event: Vec<Event> = vec![];
    let mut clipboard = clipboard::Null;
    let cursor = Cursor::Unavailable;
    let theme = Theme::Light;
    let window = wrap.size();
    let mut input = input::InputHandler::new();

    // 记录是否出现事件
    let mut instant = std::time::Instant::now(); // 屏幕刷新率控制
    let duration_per_frame = std::time::Duration::from_secs(1) / 60;
    let mut update_flag = false;
    // 用来记录是否进入息屏
    let mut last_active_time = std::time::Instant::now();
    let mut last_active_time_flag = false;
    let mut last_active_timeout = std::time::Duration::from_secs(state.config.get_arc().screen_timeout.into());
    loop {
        state.update(&push, Message::Ticker);
        for i in 0..300 {
            instant = std::time::Instant::now();

            last_active_time_flag = false;
            update_flag = false;

            // 为了防止出现死循环，这里需要先查询出所有消息，然后再调用 update
            messages.extend(pull.pull());
            messages.extend(input.poll_events().into_iter().map(Message::Pad));
            for msg in messages.drain(..) {
                if let Message::Refresh = msg{
                    update_flag = true;
                    continue;
                }
                if let Message::SetTimeout(s) = msg {
                    last_active_timeout = std::time::Duration::from_secs(s as u64);
                    state.config.set(|cfg|{
                        cfg.screen_timeout = s;
                    });
                    continue;
                }
                if let Message::Pad(k) = msg {
                    match k {
                        DPad::Power => {
                            play_wav(&mixer, DEFAULT_SOUND.device_added);
                        }
                        DPad::VolumeUp => {
                            play_wav(&mixer, DEFAULT_SOUND.audio_volume_change);
                        }
                        DPad::VolumeDown => {
                            play_wav(&mixer, DEFAULT_SOUND.audio_volume_change);
                        }
                        _ => {
                            play_wav(&mixer, DEFAULT_SOUND.message);
                        }
                    }
                    last_active_time_flag = true;
                }
                if let Message::Pad(DPad::Power) = msg {
                    return Ok(Next::Hibernate);
                } else if let Message::Launch{exec,args} = msg {
                    let mut cmd = Command::new(exec);
                    cmd.args(args);
                    return Ok(Next::Cmd(cmd)); // 跳出循环，启动新的软件
                } else {
                    state.update(&push, msg);
                    update_flag = true;
                }
            }
            if last_active_time_flag {
                last_active_time = std::time::Instant::now();
            } else if last_active_timeout.as_secs() > 0
                && last_active_time.elapsed() > last_active_timeout
            {
                return Ok(Next::Hibernate);
            }
            if i > 0 && !update_flag {
                // 如果状态没有出现变化，那么不需要重新渲染
                std::thread::sleep(duration_per_frame - instant.elapsed());
                continue;
            }
            let view = state.view();
            let mut ui = UserInterface::build(view, Size::from(window), cache, wrap.renderer());
            let _ = ui.update(
                &event,
                cursor.clone(),
                wrap.renderer(),
                &mut clipboard,
                &mut messages,
            );
            ui.draw(
                wrap.renderer(),
                &theme,
                &renderer::Style::default(),
                cursor.clone(),
            );
            wrap.present();
            cache = ui.into_cache();
        }
    }
}
pub enum Next {
    Hibernate,
    Cmd(Command),
}
struct Sound {
    message: &'static [u8],
    audio_volume_change: &'static [u8],
    bell: &'static [u8],
    camera_shutter: &'static [u8],
    complete: &'static [u8],
    device_added: &'static [u8],
    device_removed: &'static [u8],
    dialog_warning: &'static [u8],
    message_new_instant: &'static [u8],
}
const DEFAULT_SOUND: Sound = Sound {
    message: include_bytes!("../../assets/stereo/message.wav"),
    audio_volume_change: include_bytes!("../../assets/stereo/audio-volume-change.wav"),
    bell: include_bytes!("../../assets/stereo/bell.wav"),
    camera_shutter: include_bytes!("../../assets/stereo/camera-shutter.wav"),
    complete: include_bytes!("../../assets/stereo/complete.wav"),
    device_added: include_bytes!("../../assets/stereo/device-added.wav"),
    device_removed: include_bytes!("../../assets/stereo/device-removed.wav"),
    dialog_warning: include_bytes!("../../assets/stereo/dialog-warning.wav"),
    message_new_instant: include_bytes!("../../assets/stereo/message-new-instant.wav"),
};

