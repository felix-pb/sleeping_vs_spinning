mod mpsc;
mod tcp;
mod udp;

#[tokio::main]
async fn main() {
    mpsc::sleeping().await;
    mpsc::spinning().await;

    tcp::sleeping().await;
    tcp::spinning().await;

    udp::sleeping().await;
    udp::spinning().await;
}
