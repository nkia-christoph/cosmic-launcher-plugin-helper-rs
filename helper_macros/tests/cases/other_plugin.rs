use helper::*;
#[plugin]
async fn other () {
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
}
