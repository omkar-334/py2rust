//! Contains the client GUI and its associated asynchronous logic.
//!
//! The `RtpClientApp` struct implements the `eframe::App` trait for the GUI,
//! while the `async_main` function runs in a separate thread to handle all
//! network operations. They communicate via channels.

use crate::rtp::RtpPacket;
use crate::rtsp::{RtspResponse, RTSP_VERSION};
use anyhow::{anyhow, bail, Context, Result};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use eframe::egui;
use image::ImageFormat;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::Notify;
use tracing::{debug, error, info, warn};

/// Arguments required to start the client.
#[derive(Clone)]
pub struct ClientArgs {
    pub server_addr: String,
    pub server_port: u16,
    pub rtp_port: u16,
    pub video_file: String,
}

/// Represents the client's state.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ClientState {
    Init,
    Ready,
    Playing,
}

/// Messages sent from the GUI thread to the async worker thread.
enum ToAsync {
    Setup,
    Play,
    Pause,
    Teardown,
}

/// Messages sent from the async worker thread to the GUI thread.
enum FromAsync {
    UpdateState(ClientState),
    Frame(Arc<egui::ColorImage>),
    ShowError(String),
}

/// Main application struct for the eframe GUI.
struct RtpClientApp {
    args: ClientArgs,
    state: ClientState,
    texture: Option<egui::TextureHandle>,
    to_async: Sender<ToAsync>,
    from_async: Receiver<FromAsync>,
    error_message: Option<String>,
}

impl RtpClientApp {
    fn new(
        _cc: &eframe::CreationContext<'_>,
        args: ClientArgs,
        to_async: Sender<ToAsync>,
        from_async: Receiver<FromAsync>,
    ) -> Self {
        Self {
            args,
            state: ClientState::Init,
            texture: None,
            to_async,
            from_async,
            error_message: None,
        }
    }
}

impl eframe::App for RtpClientApp {
    /// Called each frame to update the GUI.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            // If the user tries to close the window, send a teardown message.
            if self.state != ClientState::Init {
                info!("Teardown on close");
                self.to_async.send(ToAsync::Teardown).ok();
            }
        }

        // Process messages from the async worker
        loop {
            match self.from_async.try_recv() {
                Ok(msg) => match msg {
                    FromAsync::UpdateState(new_state) => self.state = new_state,
                    FromAsync::Frame(color_image) => {
                        self.texture = Some(ctx.load_texture(
                            "video_frame",
                            (*color_image).clone(),
                            Default::default(),
                        ));
                    }
                    FromAsync::ShowError(err_msg) => {
                        self.error_message = Some(err_msg);
                    }
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.error_message = Some("Worker thread disconnected".to_string());
                    break;
                }
            }
        }

        egui::TopBottomPanel::top("info_panel").show(ctx, |ui| {
            ui.heading("RTSP Video Client");
            ui.horizontal(|ui| {
                ui.label(format!("Server: {}:{}", self.args.server_addr, self.args.server_port));
                ui.separator();
                ui.label(format!("File: {}", self.args.video_file));
                ui.separator();
                ui.label(format!("State: {:?}", self.state));
            });
            if let Some(err) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("ERROR: {}", err));
            }
        });

        egui::TopBottomPanel::bottom("buttons_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.add_enabled(self.state == ClientState::Init, egui::Button::new("Setup")).clicked() {
                    self.to_async.send(ToAsync::Setup).unwrap();
                }
                if ui.add_enabled(self.state == ClientState::Ready, egui::Button::new("Play")).clicked() {
                    self.to_async.send(ToAsync::Play).unwrap();
                }
                if ui.add_enabled(self.state == ClientState::Playing, egui::Button::new("Pause")).clicked() {
                    self.to_async.send(ToAsync::Pause).unwrap();
                }
                if ui.add_enabled(self.state != ClientState::Init, egui::Button::new("Teardown")).clicked() {
                    self.to_async.send(ToAsync::Teardown).unwrap();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.texture {
                ui.image(texture);
            } else {
                ui.label("No video stream yet. Press 'Setup' then 'Play'.");
            }
        });

        // Request a repaint to show the next frame
        ctx.request_repaint_after(Duration::from_millis(10));
    }
}

/// Launches the GUI and the async worker thread.
pub fn run_gui(args: ClientArgs) -> Result<()> {
    let (to_async_tx, to_async_rx) = crossbeam_channel::unbounded();
    let (from_async_tx, from_async_rx) = crossbeam_channel::unbounded();

    let args_clone = args.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        if let Err(e) = rt.block_on(async_main(args_clone, to_async_rx, from_async_tx.clone())) {
            error!("Async worker failed: {}", e);
            from_async_tx.send(FromAsync::ShowError(e.to_string())).ok();
        }
    });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "RTPClient",
        options,
        Box::new(move |cc| Box::new(RtpClientApp::new(cc, args, to_async_tx, from_async_rx))),
    )
    .map_err(|e| anyhow!("Failed to run eframe GUI: {}", e))
}

/// The main async function that handles all network I/O.
async fn async_main(
    args: ClientArgs,
    gui_rx: Receiver<ToAsync>,
    gui_tx: Sender<FromAsync>,
) -> Result<()> {
    let server_socket_addr = format!("{}:{}", args.server_addr, args.server_port);
    let mut rtsp_socket = TcpStream::connect(&server_socket_addr)
        .await
        .with_context(|| format!("Failed to connect to server at {}", server_socket_addr))?;
    info!("Connected to RTSP server at {}", server_socket_addr);

    let mut rtsp_seq = 0;
    let mut session_id = 0;
    let rtp_shutdown_notify = Arc::new(Notify::new());

    loop {
        match gui_rx.recv() {
            Ok(cmd) => {
                let should_break = handle_command(
                    cmd,
                    &mut rtsp_socket,
                    &mut rtsp_seq,
                    &mut session_id,
                    &args,
                    &gui_tx,
                    rtp_shutdown_notify.clone(),
                )
                .await?;
                if should_break {
                    break;
                }
            }
            Err(_) => {
                info!("GUI channel closed, shutting down async worker.");
                break;
            }
        }
    }
    Ok(())
}

/// Handles a single command from the GUI. Returns `true` if the loop should terminate.
async fn handle_command(
    cmd: ToAsync,
    rtsp_socket: &mut TcpStream,
    rtsp_seq: &mut u32,
    session_id: &mut u32,
    args: &ClientArgs,
    gui_tx: &Sender<FromAsync>,
    rtp_shutdown: Arc<Notify>,
) -> Result<bool> {
    *rtsp_seq += 1;
    let cseq = *rtsp_seq;

    let request_str = match cmd {
        ToAsync::Setup => {
            format!(
                "SETUP {} {}\r\nCSeq: {}\r\nTransport: RTP/UDP; client_port={}\r\n\r\n",
                args.video_file,
                RTSP_VERSION,
                cseq,
                args.rtp_port
            )
        }
        ToAsync::Play => {
            format!(
                "PLAY {} {}\r\nCSeq: {}\r\nSession: {}\r\n\r\n",
                args.video_file, RTSP_VERSION, cseq, *session_id
            )
        }
        ToAsync::Pause => {
            format!(
                "PAUSE {} {}\r\nCSeq: {}\r\nSession: {}\r\n\r\n",
                args.video_file, RTSP_VERSION, cseq, *session_id
            )
        }
        ToAsync::Teardown => {
            format!(
                "TEARDOWN {} {}\r\nCSeq: {}\r\nSession: {}\r\n\r\n",
                args.video_file, RTSP_VERSION, cseq, *session_id
            )
        }
    };

    debug!("Sending RTSP request:\n{}", request_str);
    rtsp_socket.write_all(request_str.as_bytes()).await?;

    let mut buffer = [0; 1024];
    let n = rtsp_socket.read(&mut buffer).await?;
    if n == 0 {
        bail!("Server closed the connection unexpectedly");
    }
    let response_str = std::str::from_utf8(&buffer[..n])?;
    debug!("Received RTSP response:\n{}", response_str);

    let response = RtspResponse::parse(response_str)?;
    if response.cseq != cseq {
        warn!("Received response with mismatched CSeq. Expected {}, got {}", cseq, response.cseq);
    }
    if response.status_code != 200 {
        bail!("Server returned error: {} {}", response.status_code, response_str.lines().next().unwrap_or(""));
    }

    match cmd {
        ToAsync::Setup => {
            *session_id = response.session_id;
            gui_tx.send(FromAsync::UpdateState(ClientState::Ready))?;
        }
        ToAsync::Play => {
            gui_tx.send(FromAsync::UpdateState(ClientState::Playing))?;
            let rtp_socket = UdpSocket::bind(format!("0.0.0.0:{}", args.rtp_port)).await?;
            tokio::spawn(listen_rtp(rtp_socket, gui_tx.clone(), rtp_shutdown));
        }
        ToAsync::Pause => {
            gui_tx.send(FromAsync::UpdateState(ClientState::Ready))?;
            rtp_shutdown.notify_one();
        }
        ToAsync::Teardown => {
            gui_tx.send(FromAsync::UpdateState(ClientState::Init))?;
            rtp_shutdown.notify_one();
            return Ok(true); // Terminate loop
        }
    }

    Ok(false)
}

/// Task to listen for RTP packets on a UDP socket.
async fn listen_rtp(
    socket: UdpSocket,
    gui_tx: Sender<FromAsync>,
    shutdown: Arc<Notify>,
) -> Result<()> {
    info!("RTP listener started on {}", socket.local_addr()?);
    let mut buf = vec![0; 20480]; // Buffer for one RTP packet

    loop {
        tokio::select! {
            Ok((len, _addr)) = socket.recv_from(&mut buf) => {
                match RtpPacket::decode(&buf[..len]) {
                    Ok(packet) => {
                        // Assume JPEG payload
                        match image::load_from_memory_with_format(&packet.payload, ImageFormat::Jpeg) {
                            Ok(image) => {
                                let size = [image.width() as _, image.height() as _];
                                let image_buffer = image.to_rgba8();
                                let pixels = image_buffer.as_flat_samples();
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                                if gui_tx.send(FromAsync::Frame(Arc::new(color_image))).is_err() {
                                    break; // GUI closed
                                }
                            }
                            Err(e) => warn!("Failed to decode JPEG frame: {}", e),
                        }
                    }
                    Err(e) => warn!("Failed to decode RTP packet: {}", e),
                }
            }
            _ = shutdown.notified() => {
                info!("RTP listener shutting down.");
                break;
            }
        }
    }
    Ok(())
}