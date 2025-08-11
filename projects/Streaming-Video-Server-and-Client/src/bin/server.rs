//! Server binary for the RTSP video streamer.
//!
//! This server listens for RTSP connections from clients, manages video streaming
//! sessions, and sends video data over RTP/UDP.

use anyhow::Result;
use clap::Parser;
use rtsp_video_streamer::server_logic::ServerWorker;
use tokio::net::TcpListener;
use tracing::info;

/// RTSP Video Streamer Server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The port number for the server to listen on for RTSP connections.
    #[arg(short, long, default_value_t = 5555)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (logging)
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening for RTSP connections on {}", addr);

    loop {
        let (stream, client_addr) = listener.accept().await?;
        info!("Accepted connection from: {}", client_addr);

        // Spawn a new task for each client connection.
        tokio::spawn(async move {
            let mut worker = ServerWorker::new(stream, client_addr);
            if let Err(e) = worker.handle_connection().await {
                tracing::error!(
                    "Error handling connection from {}: {}. Connection terminated.",
                    client_addr,
                    e
                );
            } else {
                info!("Connection with {} closed gracefully.", client_addr);
            }
        });
    }
}