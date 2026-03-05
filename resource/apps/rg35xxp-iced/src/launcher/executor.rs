use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub trait Push<Message>: Clone {
    fn push(&self, m: Message);
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = Message> + Send + 'static;
}
pub trait Pull<Message> {
    fn pull(&mut self) -> Vec<Message>;
}
impl<M, T: Push<M>> Push<M> for &T {
    fn push(&self, m: M) {
        (*self).push(m);
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = M> + Send + 'static,
    {
        (*self).spawn(future);
    }
}

#[derive(Clone)]
pub struct PushWrap<Message>(Arc<Runtime>, UnboundedSender<Message>);

impl<Message: 'static + Send + Clone> Push<Message> for PushWrap<Message> {
    fn push(&self, m: Message) {
        let _ = self.1.send(m);
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = Message> + Send + 'static,
    {
        let sender = self.1.clone();

        let _ = self.0.spawn(async move {
            // 等待 Future 执行完毕
            let message = future.await;
            // 将结果发送回 UI 线程
            let _ = sender.send(message);
        });
    }
}

impl<M> From<UnboundedReceiver<M>> for PullWrap<M> {
    fn from(value: UnboundedReceiver<M>) -> Self {
        Self(value)
    }
}
impl<M> Into<UnboundedReceiver<M>> for PullWrap<M> {
    fn into(self) -> UnboundedReceiver<M> {
        self.0
    }
}
pub struct PullWrap<Message>(UnboundedReceiver<Message>);
impl<Message: 'static + Send> Pull<Message> for PullWrap<Message> {
    fn pull(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();
        loop {
            match self.0.try_recv() {
                Ok(msg) => messages.push(msg),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break, // 通道断开，理论上不应发生
            }
        }
        messages
    }
}

pub fn new<Message: Clone>() -> (PushWrap<Message>, PullWrap<Message>) {
    // 1. 创建 Tokio 的无界通道
    let (tx, rx) = unbounded_channel();

    // 2. 创建 Tokio Runtime
    // 这里使用 new_multi_thread 会自动创建工作线程池。
    // 只要这个 rt 对象不被 drop，后台线程池就会一直运行。
    let rt = Runtime::new().expect("Failed to create tokio runtime");
    (PushWrap(Arc::new(rt), tx), PullWrap(rx))
}
