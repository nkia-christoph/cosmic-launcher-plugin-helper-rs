use helper::*;
#[plugin(name="my plugin")]
async fn main () {
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
}
