use lmdb_tui::worker::Worker;

#[tokio::test]
async fn worker_processes_requests() -> anyhow::Result<()> {
    let worker = Worker::start(1, |n: u32| async move { n * 2 });

    let result = worker.request(3).await?;
    assert_eq!(result, 6);
    Ok(())
}
