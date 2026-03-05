use crate::ctrl::*;
use anyhow::anyhow;
use nix::mount::{mount, umount, MsFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::process::{ExitCode, Stdio};
use std::sync::Arc;
use std::{env, io};
use tokio::io::AsyncReadExt;
use tokio::net::UdpSocket;
use tokio::process::{ChildStderr, ChildStdout, Command};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use crate::buffer::SimpleBuffer;

#[cfg(feature = "file_logger")]
macro_rules! log {
    ($($arg:tt)*) => {{
        let _ = __log(&format!($($arg)*));
    }};
}
#[cfg(not(feature = "file_logger"))]
macro_rules! log {
    ($($arg:tt)*) => {{
        println!($($arg)*);
    }};
}

#[derive(Deserialize, Debug)]
struct Mount {
    dev: String,
    dir: String,
    typ: String,
    ops: Option<String>,
}

fn get_default_shell() -> String {
    env::var("USE_SHELL").unwrap_or("/bin/sh".to_string())
}

#[derive(Deserialize, Debug)]
struct Script {
    #[serde(default = "get_default_shell")]
    shell: String,
    cmd: String,
    wd: Option<String>,
    env: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug)]
#[serde(default)]
struct Config {
    env: HashMap<String, String>,
    mount: Vec<Mount>,
    setup: Vec<Script>,
    service: HashMap<String, Service>,
    unset: Vec<Script>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            env: Default::default(),
            mount: vec![],
            setup: vec![],
            service: Default::default(),
            unset: vec![],
        }
    }
}

enum ManagerCmd {
    Start { name: String },
    Stop { name: String },
    Restart { name: String },
    List { resp: tokio::sync::oneshot::Sender<Vec<ServiceInfo>> },
    GetLog { name: String, resp: tokio::sync::oneshot::Sender<Option<String>> },
    ChildExited { name: String, status: std::process::ExitStatus },
}

#[derive(Clone)]
struct ServiceManager {
    sender: tokio::sync::mpsc::Sender<ManagerCmd>,
}

struct RunningState {
    token: CancellationToken,
}

impl ServiceManager {
    pub fn new(configs: HashMap<String, Service>) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel(32);
        let manager_sender = tx.clone();

        tokio::spawn(async move {
            let services: HashMap<String, Service> = configs;
            let mut running: HashMap<String, RunningState> = HashMap::new();
            let mut last_status: HashMap<String, ServiceStatus> = HashMap::new();
            let mut logs: HashMap<String, Arc<Mutex<SimpleBuffer>>> = HashMap::new();

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    ManagerCmd::Start { name } => {
                        if running.contains_key(&name) {
                            log!("Service '{}' is already running", name);
                            continue;
                        }
                        let config = match services.get(&name) {
                            Some(c) => c.clone(),
                            None => {
                                log!("Service '{}' config not found", name);
                                continue;
                            }
                        };

                        let mut command = Command::new(&config.exec);
                        command.args(&config.args);
                        if let Some(wd) = &config.wd {
                            command.current_dir(wd);
                        }
                        if let Some(env) = &config.env {
                            for (key, value) in env {
                                command.env(key, value);
                            }
                        }
                        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

                        match command.spawn() {
                            Ok(mut child) => {
                                let pid = child.id().unwrap();
                                let token = CancellationToken::new();
                                let child_token = token.clone();
                                let tx_exit = manager_sender.clone();
                                let name_clone = name.clone();

                                if !logs.contains_key(&name) {
                                    logs.insert(name.clone(), Arc::new(Mutex::new(SimpleBuffer::new())));
                                }
                                let log_buffer = logs.entry(name.clone())
                                    .or_insert_with(|| Arc::new(Mutex::new(SimpleBuffer::new())));
                                let stdout = child.stdout.take().unwrap();
                                let stderr = child.stderr.take().unwrap();
                                let log_buffer_stdout = log_buffer.clone();
                                let log_buffer_stderr = log_buffer.clone();

                                tokio::spawn(async move {
                                    let mut reader = stdout;
                                    let mut buf = [0u8; 4096];
                                    while let Ok(n) = reader.read(&mut buf).await {
                                        if n == 0 { break; }
                                        log_buffer_stdout.lock().await.push(&buf[..n]);
                                    }
                                });

                                tokio::spawn(async move {
                                    let mut reader = stderr;
                                    let mut buf = [0u8; 4096];
                                    while let Ok(n) = reader.read(&mut buf).await {
                                        if n == 0 { break; }
                                        log_buffer_stderr.lock().await.push(&buf[..n]);
                                    }
                                });

                                log!("Service '{}' (PID: {}) started", name, pid);

                                tokio::spawn(async move {
                                    tokio::select! {
                                        _ = child_token.cancelled() => {
                                            let _ = child.kill().await;
                                            if let Ok(status) = child.wait().await {
                                                let _ = tx_exit.send(ManagerCmd::ChildExited { name: name_clone, status }).await;
                                            }
                                        }
                                        Ok(status) = child.wait() => {
                                            let _ = tx_exit.send(ManagerCmd::ChildExited { name: name_clone, status }).await;
                                        }
                                    }
                                });

                                let start_time = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                running.insert(name.clone(), RunningState { token });
                                last_status.insert(name, ServiceStatus::Running { pid, start_time });
                            }
                            Err(e) => {
                                log!("Failed to start service '{}': {}", name, e);
                                last_status.insert(name, ServiceStatus::Failed { reason: e.to_string() });
                            }
                        }
                    }
                    ManagerCmd::Stop { name } => {
                        if let Some(state) = running.remove(&name) {
                            state.token.cancel();
                        }
                    }
                    ManagerCmd::Restart { name } => {
                        if let Some(state) = running.remove(&name) {
                            state.token.cancel();
                        }
                        let _ = manager_sender.send(ManagerCmd::Start { name }).await;
                    }
                    ManagerCmd::List { resp } => {
                        let mut list = Vec::new();
                        for (name, config) in &services {
                            let status = last_status.get(name).cloned().unwrap_or(ServiceStatus::Stopped { exit_code: None });
                            list.push(ServiceInfo {
                                name: name.clone(),
                                status,
                                config: config.clone(),
                            });
                        }
                        let _ = resp.send(list);
                    }
                    ManagerCmd::GetLog { name, resp } => {
                        log!("get service[{}] logs:", &name);
                        if let Some(log) = logs.get(&name) {
                            let log_str = log.lock().await.to_string();
                            log!("{}", &log_str);
                            let _ = resp.send(Some(log_str));
                        } else {
                            let _ = resp.send(None);
                        }
                    }
                    ManagerCmd::ChildExited { name, status } => {
                        log!("Service '{}' exited with status: {}", name, status);
                        running.remove(&name);
                        last_status.insert(name, ServiceStatus::Stopped { exit_code: status.code() });
                    }
                }
            }
        });

        Self { sender: tx }
    }

    pub async fn start(&self, name: &str) {
        let _ = self.sender.send(ManagerCmd::Start { name: name.to_string() }).await;
    }

    pub async fn stop(&self, name: &str) {
        let _ = self.sender.send(ManagerCmd::Stop { name: name.to_string() }).await;
    }

    pub async fn restart(&self, name: &str) {
        let _ = self.sender.send(ManagerCmd::Restart { name: name.to_string() }).await;
    }

    pub async fn list(&self) -> Vec<ServiceInfo> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.sender.send(ManagerCmd::List { resp: tx }).await;
        rx.await.unwrap_or_default()
    }

    pub async fn get_log(&self, name: &str) -> Option<String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.sender.send(ManagerCmd::GetLog { name: name.to_string(), resp: tx }).await;
        rx.await.unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
enum Action {
    Reboot,
    Off,
    Exit,
}

pub fn run() -> ExitCode {
    crate::logo::display_logo();
    log!("==> Hello from rg35xxp-init");

    let config = match std::fs::read_to_string("init.toml")
        .map_err(|e| anyhow!("Failed to read config file: {}", e))
        .and_then(|s| {
            toml::from_str::<Config>(&s).map_err(|e| anyhow!("Failed to parse config file: {}", e))
        }) {
        Ok(config) => {
            log!("loading init.toml completed");
            config
        }
        Err(err) => {
            log!("loading init.toml error: {}", err);
            return ExitCode::FAILURE;
        }
    };

    for (key, value) in &config.env {
        unsafe { std::env::set_var(key, value); }
    }

    if !Path::new("/dev/pts").exists() {
        let _ = std::fs::create_dir("/dev/pts");
    }
    if let Err(err) = mount::<str, str, str, str>(Some("devpts"), "/dev/pts", Some("devpts"), parse_mount_options("rw,nosuid,noexec,relatime"), None) {
        log!("Failed to mount dev/pts: {}", err);
    }

    if !Path::new("/dev/shm").exists() {
        let _ = std::fs::create_dir("/dev/shm");
    }
    if let Err(err) = mount::<str, str, str, str>(Some("tmpfs"), "/dev/shm", Some("tmpfs"), parse_mount_options(""), None) {
        log!("Failed to mount dev/shm: {}", err);
    }

    if let Err(err) = mount::<str, str, str, str>(Some("proc"), "/proc", Some("proc"), parse_mount_options("rw,nosuid,nodev,noexec,relatime"), None) {
        log!("Failed to mount proc: {}", err);
    }

    if let Err(err) = mount::<str, str, str, str>(Some("sysfs"), "/sys", Some("sysfs"), parse_mount_options("rw,nosuid,nodev,noexec,relatime"), None) {
        log!("Failed to mount sys: {}", err);
    }

    let mut mounted: Vec<String> = vec![];
    for x in &config.mount {
        let flags = x.ops.as_ref().map(|ops| parse_mount_options(ops)).unwrap_or(MsFlags::empty());
        match mount::<str, str, str, str>(Some(x.dev.as_str()), x.dir.as_str(), Some(&x.typ.as_str()), flags, None) {
            Ok(_) => {
                log!("Mounted device '{}' to '{}' (type: {}) successfully", x.dev, x.dir, x.typ);
                mounted.push(x.dir.clone());
            }
            Err(e) => log!("Failed to mount device '{}' to '{}': {}", x.dev, x.dir, e),
        }
    }

    log!("==> Starting setup phase...");
    for x in &config.setup {
        if let Err(err) = x.run() {
            log!("Setup script '{}' failed: {}", x.cmd, err);
            return ExitCode::FAILURE;
        }
    }

    let action = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let manager = Arc::new(ServiceManager::new(config.service.clone()));
            for name in config.service.keys() {
                manager.start(name).await;
            }

            let (sender, mut receiver) = tokio::sync::mpsc::channel::<Action>(1);
            let _udp_daemon = udp(sender.clone(), manager.clone()).await;

            if std::process::id() == 1 {
                tokio::spawn(sig(sender.clone(), SignalKind::from(libc::SIGUSR1), Action::Off));
                tokio::spawn(sig(sender.clone(), SignalKind::from(libc::SIGUSR2), Action::Off));
                tokio::spawn(sig(sender.clone(), SignalKind::from(libc::SIGTERM), Action::Reboot));
                tokio::spawn(sig(sender.clone(), SignalKind::from(libc::SIGPWR), Action::Off));
                tokio::spawn(sig(sender.clone(), SignalKind::from(libc::SIGINT), Action::Reboot));
            } else {
                tokio::spawn(sig(sender.clone(), SignalKind::from(libc::SIGTERM), Action::Exit));
                tokio::spawn(sig(sender.clone(), SignalKind::from(libc::SIGINT), Action::Exit));
            }

            let action = receiver.recv().await.unwrap_or(Action::Exit);
            log!("Initiating shutdown sequence...");
            
            let services = manager.list().await;
            for info in &services {
                if let ServiceStatus::Running { .. } = info.status {
                    manager.stop(&info.name).await;
                }
            }

            loop {
                let current = manager.list().await;
                if !current.iter().any(|s| matches!(s.status, ServiceStatus::Running { .. })) {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }

            return action;
        });

    for x in &config.unset {
        let _ = x.run();
    }

    log!("==> Unmounting filesystems...");
    let mut mounts = Vec::new();
    if let Ok(content) = std::fs::read_to_string("/proc/mounts") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let mp = parts[1].to_string();
                if mp != "/" && !mp.starts_with("/dev") && mp != "/proc" && mp != "/sys" {
                    mounts.push(mp);
                }
            }
        }
    }

    for x in mounts.iter().rev() {
        if let Err(_) = umount(x.as_str()) {
            let _ = nix::mount::umount2(x.as_str(), nix::mount::MntFlags::MNT_DETACH);
        }
    }

    for x in ["/sys", "/proc", "/dev/shm", "/dev/pts"] {
        let _ = umount(x);
    }

    if std::process::id() == 1 {
        unsafe { libc::sync(); }
        match action {
            Action::Reboot => {
                log!("Rebooting...");
                unsafe { libc::reboot(libc::RB_AUTOBOOT) };
            }
            Action::Off => {
                log!("Shutting down...");
                unsafe { libc::reboot(libc::RB_POWER_OFF) };
            }
            Action::Exit => log!("Exiting..."),
        }
    }
    ExitCode::SUCCESS
}

async fn sig(sender: tokio::sync::mpsc::Sender<Action>, s: SignalKind, action: Action) {
    if let Ok(mut sig) = signal(s) {
        sig.recv().await;
        let _ = sender.send(action).await;
    }
}

impl Script {
    fn run(&self) -> anyhow::Result<()> {
        log!("Executing script: {}", self.cmd);
        let mut command = std::process::Command::new(&self.shell);
        command.stdin(Stdio::piped());
        if let Some(wd) = &self.wd { command.current_dir(wd); }
        if let Some(env) = &self.env {
            for (key, value) in env { command.env(key, value); }
        }

        let mut child = command.spawn()?;
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(self.cmd.as_bytes());
        }

        match child.wait()?.code() {
            Some(0) => Ok(()),
            Some(other) => Err(anyhow!("exit code: {}", other)),
            None => Err(anyhow!("unknown exit code")),
        }
    }
}

fn parse_mount_options(options: &str) -> MsFlags {
    let mut flags = MsFlags::empty();
    for opt in options.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
        match opt {
            "ro" => flags |= MsFlags::MS_RDONLY,
            "nosuid" => flags |= MsFlags::MS_NOSUID,
            "nodev" => flags |= MsFlags::MS_NODEV,
            "noexec" => flags |= MsFlags::MS_NOEXEC,
            "relatime" => flags |= MsFlags::MS_RELATIME,
            _ => {}
        }
    }
    flags
}

const LOG_FILE: &'static str = "init.log";
fn __log(msg: &str) -> io::Result<()> {
    let mut file = File::options().create(true).write(true).append(true).open(LOG_FILE)?;
    file.write_all(msg.as_bytes())?;
    file.write_all("\n".as_bytes())?;
    file.sync_all()?; // 注意这个非常重要，因为没有任何调试机制，只能回头查看
    Ok(())
}

async fn udp(sender: tokio::sync::mpsc::Sender<Action>, manager: Arc<ServiceManager>) -> io::Result<()> {
    let addr: SocketAddr = "127.0.0.1:7788".parse().unwrap();
    let socket = UdpSocket::bind(addr).await?;
    tokio::spawn(async move {
        let mut buf: [u8; 4096] = [0; 4096];
        loop {
            if let Ok((n, src)) = socket.recv_from(&mut buf).await {
                udp_handle(sender.clone(), manager.clone(), &socket, src, &buf[..n]).await;
            }
        }
    });
    Ok(())
}

async fn reply<T: Serialize>(socket: &UdpSocket, src: SocketAddr, data: T, ok: bool) {
    let resp = Response { ok, data: Some(data) };
    if let Ok(vec) = serde_json::to_vec(&resp) {
        let _ = socket.send_to(&vec, src).await;
    }
}

async fn udp_handle(sender: tokio::sync::mpsc::Sender<Action>, manager: Arc<ServiceManager>, socket: &UdpSocket, src: SocketAddr, data: &[u8]) {
    let Ok(ins) = serde_json::from_slice::<Instruct>(data) else { return; };
    match ins {
        Instruct::Reboot => {
            reply(socket, src, "Rebooting...", true).await;
            let _ = sender.send(Action::Reboot).await;
        }
        Instruct::PowerOff => {
            reply(socket, src, "Powering off...", true).await;
            let _ = sender.send(Action::Off).await;
        }
        Instruct::ListService => {
            let list = manager.list().await;
            reply(socket, src, list, true).await;
        }
        Instruct::RestartService { name } => {
            manager.restart(&name).await;
            reply(socket, src, "Restarted", true).await;
        }
        Instruct::StopService { name } => {
            manager.stop(&name).await;
            reply(socket, src, "Stopped", true).await;
        }
        Instruct::StartService { name } => {
            manager.start(&name).await;
            reply(socket, src, "Started", true).await;
        }
        Instruct::GetServiceLog { name } => {
            if let Some(log) = manager.get_log(&name).await {
                reply(socket, src, ServiceLog { name, log }, true).await;
            } else {
                reply::<String>(socket, src, "Service not found or no logs available".to_string(), false).await;
            }
        }
    }
}
