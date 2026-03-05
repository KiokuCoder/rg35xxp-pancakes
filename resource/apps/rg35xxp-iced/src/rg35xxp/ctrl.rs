use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use std::time::Duration;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub enum ServiceStatus {
    Running { pid: u32, start_time: u64 },
    Stopped { exit_code: Option<i32> },
    Failed { reason: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub status: ServiceStatus,
    pub config: Service,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Service {
    pub exec: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub wd: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(tag = "action", content = "payload")]
pub enum Instruct {
    Reboot,
    PowerOff,
    ListService,
    RestartService { name: String },
    StopService { name: String },
    StartService { name: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T> {
    pub ok: bool,
    pub data: Option<T>,
}

/// 客户端调用接口：向指定地址发送指令并等待响应
pub async fn call_init<T>(target: &str, instruct: Instruct) -> anyhow::Result<Response<T>> 
where T: for<'de> Deserialize<'de> 
{
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let data = serde_json::to_vec(&instruct)?;
    
    socket.send_to(&data, target).await?;

    let mut buf = [0u8; 16384]; // 增加缓冲区以应对大量的服务列表
    let (n, _) = tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf)).await??;
    
    let resp: Response<T> = serde_json::from_slice(&buf[..n])?;
    Ok(resp)
}
