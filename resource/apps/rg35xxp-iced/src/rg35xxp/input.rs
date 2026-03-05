use std::collections::HashMap;
use std::time::Duration;
use crate::launcher::pad::DPad;
use evdev::{EventType, KeyCode};
use log::{error, info};
use tokio::select;
use tokio::time::Instant;

pub(crate) struct InputHandler {
    _signal: tokio::sync::oneshot::Sender<()>,
    ch: tokio::sync::mpsc::Receiver<DPad>
}

impl InputHandler {
    pub(crate) fn new() -> Self {
        // 尝试枚举所有输入设备，找到看似是手柄或键盘的设备
        // RG35XX Plus 上通常是 gpio-keys 或者特定的 adc-keys
        let mut devices = Vec::new();
        let (tx,rx) = tokio::sync::mpsc::channel::<DPad>(128);
        let (signal_tx, signal_rx) = tokio::sync::oneshot::channel::<()>();

        for (_, device) in evdev::enumerate() {
            let name = device.name().unwrap_or("Unknown");
            info!("Found input device: {}", name);

            // 简单的启发式逻辑：如果在 RG35XX 上，通常找包含 "event" 或 "key" 的设备
            // 你可以通过 ssh 连接设备运行 `evtest` 查看具体是哪个设备
            if device.supported_keys().map_or(false, |k| {
                k.contains(KeyCode::BTN_SOUTH)
                    || k.contains(KeyCode::KEY_ENTER)
                    || k.contains(KeyCode::KEY_POWER)
            }) {
                info!("Selected input device: {}", name);
                if let Err(err) = device.set_nonblocking(true) {
                    error!("Failed to set non-blocking: {}", err);
                    continue;
                }
                devices.push(device);
            }
        }

        if devices.is_empty() {
            error!("No suitable input device found!");
        }
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all() // Enable IO and Time
                .build()
                .unwrap();
            
            rt.spawn(async move {
                for dev in devices {
                    let mut stream = match dev.into_event_stream() {
                        Ok(s) => s,
                        Err(e) => {
                            error!("Failed to create event stream: {}", e);
                            continue;
                        }
                    };
                    let sender = tx.clone();
                    
                    tokio::spawn(async move {
                        let mut held_buttons: HashMap<DPad, Instant> = HashMap::new();
                        let mut axis_map: HashMap<u16, DPad> = HashMap::new();
                        
                        // Check for repeats every 20ms
                        let mut interval = tokio::time::interval(Duration::from_millis(20));
                        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                        const REPEAT_DELAY: Duration = Duration::from_millis(400);
                        const REPEAT_INTERVAL: Duration = Duration::from_millis(100);

                        loop {
                            select! {
                                _ = interval.tick() => {
                                    let now = Instant::now();
                                    for (&btn, next_time) in held_buttons.iter_mut() {
                                        if now >= *next_time {
                                            let _ = sender.send(btn).await;
                                            *next_time = now + REPEAT_INTERVAL;
                                        }
                                    }
                                }
                                res = stream.next_event() => {
                                    let ev = match res {
                                        Ok(ev) => ev,
                                        Err(e) => {
                                            error!("Device stream error: {}", e);
                                            break;
                                        }
                                    };

                                    if ev.event_type() == EventType::ABSOLUTE {
                                        let code = ev.code();
                                        // 16: HAT0X (Left/Right), 17: HAT0Y (Up/Down)
                                        if code == 16 || code == 17 {
                                            // Remove previous button for this axis
                                            if let Some(prev) = axis_map.remove(&code) {
                                                held_buttons.remove(&prev);
                                            }

                                            // Determine new button
                                            let new_btn = match (code, ev.value()) {
                                                (16, -1) => Some(DPad::Left),
                                                (16, 1) => Some(DPad::Right),
                                                (17, -1) => Some(DPad::Up),
                                                (17, 1) => Some(DPad::Down),
                                                _ => None
                                            };

                                            if let Some(btn) = new_btn {
                                                let _ = sender.send(btn).await;
                                                held_buttons.insert(btn, Instant::now() + REPEAT_DELAY);
                                                axis_map.insert(code, btn);
                                            }
                                        }
                                    } else if ev.event_type() == EventType::KEY {
                                        let key = match ev.code() {
                                            114 => Some(DPad::VolumeDown),
                                            115 => Some(DPad::VolumeUp),
                                            116 => Some(DPad::Power),
                                            304 => Some(DPad::A),
                                            305 => Some(DPad::B),
                                            306 => Some(DPad::Y),
                                            307 => Some(DPad::X),
                                            308 => Some(DPad::L1),
                                            309 => Some(DPad::R1),
                                            310 => Some(DPad::Select),
                                            311 => Some(DPad::Start),
                                            312 => Some(DPad::Menu),
                                            314 => Some(DPad::L2),
                                            315 => Some(DPad::R2),
                                            _ => None
                                        };

                                        if let Some(k) = key {
                                            match ev.value() {
                                                1 => { // Press
                                                    let _ = sender.send(k).await;
                                                    held_buttons.insert(k, Instant::now() + REPEAT_DELAY);
                                                }
                                                0 => { // Release
                                                    held_buttons.remove(&k);
                                                }
                                                _ => {} // Ignore autorepeat from kernel
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                }
            });
            let _ = rt.block_on(async {
                signal_rx.await.ok();
            });
        });

        Self { ch:rx, _signal: signal_tx }
    }

    pub(crate) fn poll_events(&mut self) -> Vec<DPad> {
        let mut events: Vec<DPad> = Vec::new();
        while let Ok(event) = self.ch.try_recv() {
            events.push(event);
        }
        events
    }
}

pub fn wait_power_key() {
    let Some(mut device) = evdev::enumerate()
        .into_iter()
        .map(|(_, d)| d)
        .filter(|d| d.supported_keys().map_or(false, |k| k.contains(KeyCode::KEY_POWER)))
        .next() else {
        return;
    };
    if let Err(err) = device.set_nonblocking(false) {
        error!("Failed to set non-blocking: {}", err);
        return;
    }
    while let Ok(events) = device.fetch_events() {
        if events.filter(|ev| ev.event_type() == EventType::KEY)
            .filter(|ev| ev.code() == 116 && ev.value() == 0).next().is_some() {
            return;
        }
    }
}
pub fn wait_power_key_timeout(timeout:Duration)->bool {
    let Some(device) = evdev::enumerate()
        .into_iter()
        .map(|(_, d)| d)
        .filter(|d| d.supported_keys().map_or(false, |k| k.contains(KeyCode::KEY_POWER)))
        .next() else {
        return false;
    };
    if let Err(err) = device.set_nonblocking(true) {
        error!("Failed to set non-blocking: {}", err);
        return false;
    }
    let rt = tokio::runtime::Builder::new_current_thread().enable_io().enable_time().build().unwrap();

    rt.block_on(async move {
        let mut stream = device.into_event_stream().unwrap();
        loop {
            select! {
                _ = tokio::time::sleep(timeout) => {
                    info!("超时进入睡眠");
                    return true;
                }
                event = stream.next_event() => {
                    match event {
                        Ok(ev) => {
                            if ev.event_type() == EventType::KEY && ev.code() == 116 && ev.value() == 0 {
                                // 用户再次按下电源键，结束休眠
                                return false;
                            }
                        }
                        Err(err) => {
                            error!("未知系统错误: {}", err);
                            return false;
                        }
                    }
                }
            }
        }
    })
}
