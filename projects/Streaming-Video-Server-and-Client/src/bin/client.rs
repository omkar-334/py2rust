//! Client binary for the RTSP video streamer.
//!
//! This client provides a GUI to connect to an RTSP server, control video
//! playback (Setup, Play, Pause, Teardown), and display the received video stream.

use anyhow::Result;
use clap::Parser;
use rtsp_video_streamer::client_logic::{run_gui, ClientArgs};

/// RTSP Video Streamer Client
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The server's address or hostname.
    #[arg(short, long)]
    server_addr: String,

    /// The server's port number for RTSP connections.
    #[arg(short, long)]
    server_port: u16,

    /// The local port number for receiving RTP packets.
    #[arg(short, long)]
    rtp_port: u16,

    /// The name of the video file to request from the server.
    #[arg(short, long)]
    video_file: String,
}

fn main() -> Result<()> {
    // Initialize tracing (logging)
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let client_args = ClientArgs {
        server_addr: args.server_addr,
        server_port: args.server_port,
        rtp_port: args.rtp_port,
        video_file: args.video_file,
    };

    // The GUI needs to run on the main thread.
    run_gui(client_args)?;

    Ok(())
}