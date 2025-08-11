//! Provides a `VideoStream` struct to read video frames from a file.
//!
//! The video file is expected to have a custom format where each frame is
//! prefixed by a 5-byte ASCII string representing the frame's length.

use std::fs::File;
use std::io::{self, BufReader, Read};
use tracing::warn;

/// Reads video frames from a file with a custom format.
pub struct VideoStream {
    reader: BufReader<File>,
    frame_num: u32,
}

impl VideoStream {
    /// Creates a new `VideoStream` by opening the given file.
    pub fn new(filename: &str) -> io::Result<Self> {
        let file = File::open(filename)?;
        Ok(Self {
            reader: BufReader::new(file),
            frame_num: 0,
        })
    }

    /// Reads the next frame from the video file.
    /// Returns `Ok(None)` on EOF.
    pub fn next_frame(&mut self) -> io::Result<Option<Vec<u8>>> {
        let mut len_buf = [0u8; 5];
        match self.reader.read_exact(&mut len_buf) {
            Ok(_) => (),
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None), // Clean EOF
            Err(e) => return Err(e),
        }

        let len_str = match std::str::from_utf8(&len_buf) {
            Ok(s) => s,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
        };

        let frame_len = match len_str.parse::<usize>() {
            Ok(len) => len,
            Err(e) => {
                warn!("Invalid frame length string: '{}'", len_str);
                return Err(io::Error::new(io::ErrorKind::InvalidData, e));
            }
        };

        let mut frame_data = vec![0u8; frame_len];
        self.reader.read_exact(&mut frame_data)?;

        self.frame_num += 1;
        Ok(Some(frame_data))
    }

    /// Returns the number of the last frame that was read.
    pub fn frame_number(&self) -> u32 {
        self.frame_num
    }
}