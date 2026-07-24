
use std::path::PathBuf;


#[derive(Debug)]
pub enum WriteOp {
    Write { path: PathBuf, content: Vec<u8> },
    Append { path: PathBuf, content: Vec<u8> },
    Delete { path: PathBuf },
    Flush { done: tokio::sync::oneshot::Sender<()> },
}

pub struct WriteChannel {
    sender: tokio::sync::mpsc::UnboundedSender<WriteOp>,
}

impl WriteChannel {
    pub fn new() -> (Self, tokio::sync::mpsc::UnboundedReceiver<WriteOp>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        (Self { sender }, receiver)
    }

    pub fn write(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String> {
        self.sender
            .send(WriteOp::Write { path, content })
            .map_err(|_| String::from("channel closed"))
    }

    pub fn append(&self, path: PathBuf, content: Vec<u8>) -> Result<(), String> {
        self.sender
            .send(WriteOp::Append { path, content })
            .map_err(|_| String::from("channel closed"))
    }

    pub fn delete(&self, path: PathBuf) -> Result<(), String> {
        self.sender
            .send(WriteOp::Delete { path })
            .map_err(|_| String::from("channel closed"))
    }

    pub async fn flush(&self) -> Result<(), String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(WriteOp::Flush { done: tx })
            .map_err(|_| "channel closed".into())?;
        rx.await.map_err(|_| "flush failed".into())
    }

    pub fn spawn_consumer(
        mut receiver: tokio::sync::mpsc::UnboundedReceiver<WriteOp>,
        _project_root: PathBuf,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(op) = receiver.recv().await {
                let result = tokio::task::spawn_blocking(move || match op {
                    WriteOp::Write { path, content } => {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Err(e) = std::fs::write(&path, &content) {
                            tracing::error!("WriteChannel write error {}: {}", path.display(), e);
                        }
                    }
                    WriteOp::Append { path, content } => {
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        if let Ok(mut file) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&path)
                        {
                            use std::io::Write;
                            if let Err(e) = file.write_all(&content) {
                                tracing::error!(
                                    "WriteChannel append error {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                    WriteOp::Delete { path } => {
                        if let Err(e) = std::fs::remove_file(&path) {
                            if e.kind() != std::io::ErrorKind::NotFound {
                                tracing::error!(
                                    "WriteChannel delete error {}: {}",
                                    path.display(),
                                    e
                                );
                            }
                        }
                    }
                    WriteOp::Flush { done } => {
                        let _ = done.send(());
                    }
                })
                .await;
                if let Err(e) = result {
                    tracing::error!("WriteChannel spawn_blocking panicked: {}", e);
                }
            }
        })
    }
}
