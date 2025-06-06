use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

/// Message sent to the worker containing the request and a channel to send back the response.
struct Job<Req, Res> {
    req: Req,
    resp: oneshot::Sender<Res>,
}

/// Handle to a background worker processing requests asynchronously.
pub struct Worker<Req, Res> {
    tx: mpsc::Sender<Job<Req, Res>>,
}

impl<Req, Res> Worker<Req, Res>
where
    Req: Send + 'static,
    Res: Send + 'static,
{
    /// Spawn a new worker. The given handler will be called for every request.
    pub fn start<F, Fut>(buffer: usize, mut handler: F) -> Self
    where
        F: FnMut(Req) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Res> + Send + 'static,
    {
        let (tx, mut rx) = mpsc::channel::<Job<Req, Res>>(buffer);
        tokio::spawn(async move {
            while let Some(Job { req, resp }) = rx.recv().await {
                let result = handler(req).await;
                let _ = resp.send(result);
            }
        });
        Self { tx }
    }

    /// Send a request to the worker and wait for the response.
    pub async fn request(&self, req: Req) -> Result<Res> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(Job { req, resp: tx })
            .await
            .map_err(|e| anyhow::anyhow!("send error: {e}"))?;
        let res = rx.await.map_err(|e| anyhow::anyhow!("recv error: {e}"))?;
        Ok(res)
    }
}
