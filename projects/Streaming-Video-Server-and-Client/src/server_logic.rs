//! Contains the core logic for the RTSP server worker.
//!
//! `ServerWorker` handles a single client connection, processing RTSP requests
//! and managing the video streaming state machine.

use crate::rtp::RtpPacket;
use crate::rtsp::{RequestType, RtspRequest, RTSP_VERSION};
use crate::video_stream::VideoStream;
use anyhow::{bail, Context, Result};
use bytes::Bytes;
use rand::Rng;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Notify;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Represents the state of a client session.
#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    Init,
    Ready,
    Playing,
}

/// Manages a single client connection.
pub struct ServerWorker {
    state: State,
    stream: TcpStream,
    client_addr: SocketAddr,
    session_id: u32,
    video_stream: Option<VideoStream>,
    rtp_socket: Option<Arc<UdpSocket>>,
    rtp_dest_port: Option<u16>,
    rtp_shutdown_notify: Arc<Notify>,
}

impl ServerWorker {
    /// Creates a new `ServerWorker` for a given TCP stream and client address.
    pub fn new(stream: TcpStream, client_addr: SocketAddr) -> Self {
        Self {
            state: State::Init,
            stream,
            client_addr,
            session_id: 0,
            video_stream: None,
            rtp_socket: None,
            rtp_dest_port: None,
            rtp_shutdown_notify: Arc::new(Notify::new()),
        }
    }

    /// The main loop to handle a client connection.
    /// Reads and processes RTSP requests until the connection is closed or an error occurs.
    pub async fn handle_connection(&mut self) -> Result<()> {
        let mut buffer = [0; 1024];
        loop {
            tokio::select! {
                // Read data from the TCP stream
                read_result = self.stream.read(&mut buffer) => {
                    let n = read_result.context("Failed to read from TCP stream")?;
                    if n == 0 {
                        info!("Client {} disconnected.", self.client_addr);
                        return Ok(()); // Connection closed
                    }

                    let request_str = std::str::from_utf8(&buffer[..n])
                        .context("Received invalid UTF-8 data")?;

                    debug!("Received RTSP request:\n{}", request_str);

                    match RtspRequest::parse(request_str) {
                        Ok(request) => {
                            if let Err(e) = self.process_request(request).await {
                                error!("Error processing request: {}. Sending 500 error.", e);
                                self.reply_rtsp(500, "Connection error", 0).await?;
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse RTSP request: {}", e);
                            self.reply_rtsp(400, "Bad Request", 0).await?;
                        }
                    }
                }
                // Listen for shutdown signal to break the loop
                _ = self.rtp_shutdown_notify.notified() => {
                    info!("Teardown requested. Closing connection with {}.", self.client_addr);
                    return Ok(());
                }
            }
        }
    }

    /// Processes a parsed `RtspRequest`.
    async fn process_request(&mut self, req: RtspRequest) -> Result<()> {
        match req.request_type {
            RequestType::Setup => self.handle_setup(req).await,
            RequestType::Play => self.handle_play(req).await,
            RequestType::Pause => self.handle_pause(req).await,
            RequestType::Teardown => self.handle_teardown(req).await,
        }
    }

    /// Handles a SETUP request.
    async fn handle_setup(&mut self, req: RtspRequest) -> Result<()> {
        if self.state != State::Init {
            bail!("Received SETUP request in invalid state: {:?}", self.state);
        }
        info!("Processing SETUP for file: {}", req.filename);

        match VideoStream::new(&req.filename) {
            Ok(video_stream) => {
                self.video_stream = Some(video_stream);
                self.state = State::Ready;
                self.session_id = rand::thread_rng().gen_range(100000..=999999);
                self.rtp_dest_port = req.client_port;

                info!(
                    "Session {} created. Ready to stream {} to port {:?}.",
                    self.session_id,
                    req.filename,
                    self.rtp_dest_port.unwrap_or(0)
                );
                self.reply_rtsp(200, "OK", req.cseq).await?;
            }
            Err(_) => {
                error!("File not found: {}", req.filename);
                self.reply_rtsp(404, "File Not Found", req.cseq).await?;
            }
        }
        Ok(())
    }

    /// Handles a PLAY request.
    async fn handle_play(&mut self, req: RtspRequest) -> Result<()> {
        if self.state != State::Ready {
            bail!("Received PLAY request in invalid state: {:?}", self.state);
        }
        if req.session_id != Some(self.session_id) {
            bail!("Invalid session ID in PLAY request");
        }
        info!("Processing PLAY for session {}", self.session_id);

        let rtp_socket = UdpSocket::bind("0.0.0.0:0").await?;
        self.rtp_socket = Some(Arc::new(rtp_socket));
        self.state = State::Playing;

        self.reply_rtsp(200, "OK", req.cseq).await?;

        // Spawn a task to send RTP packets
        let rtp_task_handle = tokio::spawn(Self::send_rtp(
            self.video_stream.take().unwrap(), // Move video_stream to the task
            self.rtp_socket.clone().unwrap(),
            SocketAddr::new(self.client_addr.ip(), self.rtp_dest_port.unwrap()),
            self.rtp_shutdown_notify.clone(),
        ));

        // Spawn a task to manage the RTP task's lifecycle
        let shutdown_notify = self.rtp_shutdown_notify.clone();
        tokio::spawn(async move {
            if let Err(e) = rtp_task_handle.await {
                error!("RTP sending task failed: {:?}", e);
            }
            // Put the video stream back if the task finishes (e.g., on PAUSE)
            // This part is complex and omitted for simplicity. A channel would be better.
            // For now, PAUSE will stop sending but PLAY again won't resume from where it left off.
            shutdown_notify.notify_one();
        });

        Ok(())
    }

    /// Handles a PAUSE request.
    async fn handle_pause(&mut self, req: RtspRequest) -> Result<()> {
        if self.state != State::Playing {
            bail!("Received PAUSE request in invalid state: {:?}", self.state);
        }
        if req.session_id != Some(self.session_id) {
            bail!("Invalid session ID in PAUSE request");
        }
        info!("Processing PAUSE for session {}", self.session_id);

        self.state = State::Ready;
        self.rtp_shutdown_notify.notify_one(); // Signal the RTP sender to stop
        self.reply_rtsp(200, "OK", req.cseq).await?;
        Ok(())
    }

    /// Handles a TEARDOWN request.
    async fn handle_teardown(&mut self, req: RtspRequest) -> Result<()> {
        if req.session_id != Some(self.session_id) {
            bail!("Invalid session ID in TEARDOWN request");
        }
        info!("Processing TEARDOWN for session {}", self.session_id);

        self.rtp_shutdown_notify.notify_one(); // Signal shutdown
        self.reply_rtsp(200, "OK", req.cseq).await?;
        Ok(())
    }

    /// Sends an RTSP reply to the client.
    async fn reply_rtsp(&mut self, code: u16, status: &str, cseq: u32) -> Result<()> {
        let reply = format!(
            "{} {} {}\r\nCSeq: {}\r\nSession: {}\r\n\r\n",
            RTSP_VERSION, code, status, cseq, self.session_id
        );
        self.stream.write_all(reply.as_bytes()).await?;
        debug!("Sent RTSP reply:\n{}", reply);
        Ok(())
    }

    /// Task to send RTP packets for a video stream.
    async fn send_rtp(
        mut video_stream: VideoStream,
        rtp_socket: Arc<UdpSocket>,
        dest_addr: SocketAddr,
        shutdown: Arc<Notify>,
    ) -> Result<()> {
        info!("RTP streaming started to {}", dest_addr);
        let mut rtp_interval = interval(Duration::from_millis(50)); // ~20 FPS

        loop {
            tokio::select! {
                _ = rtp_interval.tick() => {
                    match video_stream.next_frame() {
                        Ok(Some(frame_data)) => {
                            let frame_num = video_stream.frame_number();
                            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u32;

                            // MJPEG payload type is 26
                            let rtp_packet = RtpPacket::new(26, frame_num as u16, timestamp, 0, Bytes::from(frame_data));
                            let packet_bytes = rtp_packet.encode();

                            if let Err(e) = rtp_socket.send_to(&packet_bytes, dest_addr).await {
                                error!("Failed to send RTP packet: {}", e);
                                break;
                            }
                        }
                        Ok(None) => {
                            info!("End of video stream reached.");
                            break; // End of file
                        }
                        Err(e) => {
                            error!("Error reading video frame: {}", e);
                            break;
                        }
                    }
                }
                _ = shutdown.notified() => {
                    info!("RTP streaming paused or stopped.");
                    break;
                }
            }
        }
        info!("RTP streaming to {} finished.", dest_addr);
        Ok(())
    }
}