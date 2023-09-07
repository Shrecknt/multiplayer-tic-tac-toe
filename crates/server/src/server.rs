use tokio::net::TcpListener;

pub async fn start_server(hostname: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket = TcpListener::bind(hostname).await?;

    loop {
        let stream = socket.accept().await?;
        tokio::spawn(async move {
            drop(stream);
            // handle_player_stream(stream).await.unwrap();
        });
    }
}
