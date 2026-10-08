use crate::error::{AppError, AppResult};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tokio::sync::{Semaphore, SemaphorePermit};

static DOWNLOADS: Semaphore = Semaphore::const_new(4);
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub struct Download {
    pub path: PathBuf,
    pub size: u64,
    _permit: SemaphorePermit<'static>,
}
impl Drop for Download {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub async fn download(
    client: &reqwest::Client,
    url: &str,
    directory: &Path,
    limit: u64,
) -> AppResult<Download> {
    let permit = DOWNLOADS
        .acquire()
        .await
        .map_err(|_| AppError::network("下载队列已关闭"))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::network(format!("下载失败：{e}")))?
        .error_for_status()
        .map_err(|e| AppError::network(format!("下载响应失败：{e}")))?;
    save_response(response, directory, limit, permit).await
}

async fn save_response(
    mut response: reqwest::Response,
    directory: &Path,
    limit: u64,
    permit: SemaphorePermit<'static>,
) -> AppResult<Download> {
    if response.content_length().is_some_and(|size| size > limit) {
        return Err(AppError::business(3004, "下载超过字节预算"));
    }
    let path = directory.join(format!(
        ".download-{}-{}.partial",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file: File = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let mut result = Download {
        path,
        size: 0,
        _permit: permit,
    };
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| AppError::network(format!("读取下载失败：{e}")))?
    {
        let next = result
            .size
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| AppError::business(3004, "下载字节数溢出"))?;
        if next > limit {
            return Err(AppError::business(3004, "下载超过字节预算"));
        }
        file.write_all(&chunk)?;
        result.size = next;
    }
    file.flush()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Read,
        net::TcpListener,
        time::{SystemTime, UNIX_EPOCH},
    };
    #[tokio::test]
    async fn rejects_chunked_overflow_and_removes_partial_file() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            std::time::Instant::now() < deadline,
                            "Fixture server did not receive a connection"
                        );
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(error) => panic!("Fixture accept failed: {error}"),
                }
            };
            socket
                .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                .unwrap();
            let mut request = [0u8; 4096];
            let _ = socket.read(&mut request);
            let _ = socket.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\nabcd\r\n4\r\nefgh\r\n4\r\nijkl\r\n0\r\n\r\n");
        });
        let directory = std::env::temp_dir().join(format!(
            "isle-download-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let result = download(
            &reqwest::Client::builder()
                .no_proxy()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap(),
            &format!("http://{address}"),
            &directory,
            8,
        )
        .await;
        assert!(matches!(result, Err(AppError::Business { code: 3004, .. })));
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 0);
        std::fs::remove_dir(&directory).unwrap();
        server.join().unwrap();
    }
}
