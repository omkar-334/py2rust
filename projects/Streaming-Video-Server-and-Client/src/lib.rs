//! # RTSP Video Streamer Library
//!
//! This crate contains the shared logic for the RTSP video streaming
//! server and client, including data structures for RTP and RTSP,
//! video file handling, and the core logic for both the client and server.

pub mod client_logic;
pub mod rtp;
pub mod rtsp;
pub mod server_logic;
pub mod video_stream;