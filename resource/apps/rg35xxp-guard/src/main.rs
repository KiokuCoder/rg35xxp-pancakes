use libc::Ioctl;
use std::fs::File;
use std::os::fd::{AsRawFd, RawFd};
use std::sync::atomic;
use std::time::Duration;
use std::io;

use evdev::{EventType, KeyCode};
use std::env;
use std::process::{ExitCode, ExitStatus, Stdio};
use tokio::process::Command;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    // Check arguments
    if args.len() < 2 {
        eprintln!("Usage: {} <command> [args...]", args[0]);
        return ExitCode::FAILURE;
    }
    let backlight = match Backlight::new(0) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to open backlight device (/dev/disp): {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Target command and arguments
    let target_command = &args[1];
    let target_args = &args[2..];

    // Spawn child process
    let mut child = match Command::new(target_command)
        .args(target_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Failed to start command '{}': {}", target_command, e);
            return ExitCode::FAILURE;
        }
    };
    
    let pid = match child.id() {
        Some(id) => id,
        None => {
            eprintln!("Failed to get child process PID");
            return ExitCode::FAILURE;
        }
    };

    let Some(device) = evdev::enumerate()
        .into_iter()
        .map(|(_, d)| d)
        .filter(|d| {
            d.supported_keys()
                .map_or(false, |k| k.contains(KeyCode::KEY_POWER))
        })
        .next()
    else {
        eprintln!("Failed to find power button device");
        return ExitCode::FAILURE;
    };
    if let Err(err) = device.set_nonblocking(true) {
        eprintln!("Failed to set non-blocking: {}", err);
        return ExitCode::FAILURE;
    }
    let Ok(mut stream) = device.into_event_stream().inspect_err(|err| {
        eprintln!("Operation failed: {}", err);
    }) else {
        return ExitCode::FAILURE;
    };
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Event>(1);
    
    let tx_exit = tx.clone();
    tokio::spawn(async move {
        if let Ok(status) = child.wait().await {
            let _ = tx_exit.send(Event::Exit(status)).await;
        }
    });

    let tx_power = tx.clone();
    tokio::spawn(async move {
        loop {
            match stream.next_event().await {
                Ok(ev) => {
                    if ev.event_type() == EventType::KEY
                        && ev.code() == KeyCode::KEY_POWER.0
                        && ev.value() == 0
                    {
                        let _ = tx_power.send(Event::Power).await;
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read input device: {}", e);
                    break;
                }
            }
        }
    });

    let tx_ctrlc = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(_) = tokio::signal::ctrl_c().await {
                if let Err(_) = tx_ctrlc.send(Event::CtrlC).await {
                    break;
                }
            } else {
                break;
            }
        }
    });

    // Get sleep timeout from environment or default to 300s
    let sleep_timeout = env::var("SLEEP_TIMEOUT")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(300);

    let mut paused = false;
    let mut paused_gen = 0;
    let mut last_wake = std::time::Instant::now() - Duration::from_secs(1);
    while let Some(event) = rx.recv().await {
        match event {
            Event::Power => {
                if last_wake.elapsed() < Duration::from_millis(500) {
                    continue;
                }
                if paused {
                    paused = false;
                    paused_gen += 1;
                    let _ = backlight.backlight_on();
                    let _ = resume(pid);
                } else {
                    paused = true;
                    paused_gen += 1;
                    let current_gen = paused_gen;
                    let tx_timeout = tx.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(sleep_timeout)).await;
                        let _ = tx_timeout.send(Event::Timeout(current_gen)).await;
                    });
                    let _ = pause(pid);
                    let _ = backlight.backlight_off();
                }
            }
            Event::Exit(status) => {
                return if let Some(code) = status.code() {
                    ExitCode::from(code as u8)
                } else {
                    ExitCode::FAILURE
                };
            }
            Event::Timeout(g) => {
                if !paused || g != paused_gen {
                    continue;
                }
                // Suspend system, blocks until wake
                if let Err(e) = std::fs::write("/sys/power/state", "mem") {
                    eprintln!("Failed to write /sys/power/state: {}", e);
                }
                // System waked up
                last_wake = std::time::Instant::now();
                paused = false;
                paused_gen += 1;
                let _ = backlight.backlight_on();
                let _ = resume(pid);
            }
            Event::CtrlC => {
                // Forward SIGINT to child
                let _ = resume(pid);
                unsafe { libc::kill(pid as i32, libc::SIGINT) };
            }
        }
    }
    return ExitCode::FAILURE;
}
enum Event {
    Power,
    Exit(ExitStatus),
    Timeout(u32),
    CtrlC,
}
const _DISP_LCD_SET_BRIGHTNESS: Ioctl = 0x102;
const _DISP_LCD_GET_BRIGHTNESS: Ioctl = 0x103;
const DISP_LCD_BACKLIGHT_ENABLE: Ioctl = 0x104;
const DISP_LCD_BACKLIGHT_DISABLE: Ioctl = 0x105;
const _BRIGHTNESS_MAX: u32 = 255;
pub struct Backlight {
    _file: File,
    fd: RawFd,
    channel: u32,
    enabled: atomic::AtomicBool,
}
impl Backlight {
    fn new(channel: u32) -> std::io::Result<Self> {
        let file = File::options().read(true).write(true).open("/dev/disp")?;
        let fd = file.as_raw_fd();
        Ok(Self {
            _file: file,
            fd,
            channel,
            enabled: atomic::AtomicBool::new(true),
        })
    }

    fn _set_channel(&mut self, channel: u32) -> std::io::Result<()> {
        self.channel = channel;
        Ok(())
    }

    fn _get_brightness(&self) -> std::io::Result<u32> {
        // Get current brightness
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        let ret = unsafe { libc::ioctl(self.fd, _DISP_LCD_GET_BRIGHTNESS, args.as_mut_ptr()) };
        if ret < 0 {
            return Err(io::Error::last_os_error().into());
        }
        self.enabled.store(ret > 0, atomic::Ordering::Relaxed);
        Ok(ret as u32) // Use ret directly as brightness value
    }

    fn _set_brightness(&self, brightness: u32) -> std::io::Result<()> {
        if brightness > _BRIGHTNESS_MAX {
            return Err(std::io::Error::other("brightness out of range"));
        }
        // Set brightness
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        args[1] = brightness as libc::c_ulong;
        args[2] = 0;
        unsafe {
            if libc::ioctl(self.fd, _DISP_LCD_SET_BRIGHTNESS, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error().into());
            }
        }
        self.enabled
            .store(brightness > 0, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn backlight_on(&self) -> std::io::Result<()> {
        // Enable backlight
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        unsafe {
            if libc::ioctl(self.fd, DISP_LCD_BACKLIGHT_ENABLE, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        self.enabled.store(true, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn backlight_off(&self) -> std::io::Result<()> {
        let mut args: [libc::c_ulong; 3] = [self.channel as libc::c_ulong, 0, 0];
        // Disable backlight
        unsafe {
            if libc::ioctl(self.fd, DISP_LCD_BACKLIGHT_DISABLE, args.as_mut_ptr()) < 0 {
                return Err(io::Error::last_os_error());
            }
        }
        self.enabled.store(false, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn _get_backlight(&self) -> bool {
        self.enabled.load(atomic::Ordering::Relaxed)
    }
}

fn pause(pid: u32) -> std::io::Result<()> {
    if unsafe { libc::kill(pid as i32, libc::SIGSTOP) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
fn resume(pid: u32) -> std::io::Result<()> {
    if unsafe { libc::kill(pid as i32, libc::SIGCONT) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
