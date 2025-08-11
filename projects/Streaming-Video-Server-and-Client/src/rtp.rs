//! Handles RTP packet encoding and decoding.
//!
//! The `RtpPacket` struct represents an RTP packet with methods to serialize
//! it for sending over the network and deserialize it from a received byte stream.

use anyhow::{anyhow, Result};
use bytes::Bytes;

const HEADER_SIZE: usize = 12;
const RTP_VERSION: u8 = 2;

/// Represents an RTP packet.
#[derive(Debug, Clone)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub cc: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub payload: Bytes,
}

impl RtpPacket {
    /// Creates a new RTP packet with a given payload and metadata.
    pub fn new(
        payload_type: u8,
        sequence_number: u16,
        timestamp: u32,
        ssrc: u32,
        payload: Bytes,
    ) -> Self {
        Self {
            version: RTP_VERSION,
            padding: false,
            extension: false,
            cc: 0,
            marker: false, // Marker bit is 0 for MJPEG, set to 1 for last packet of a frame if needed
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            payload,
        }
    }

    /// Encodes the RTP packet into a byte vector for transmission.
    pub fn encode(&self) -> Vec<u8> {
        let mut header = [0u8; HEADER_SIZE];

        header[0] = (self.version << 6)
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | self.cc;
        header[1] = ((self.marker as u8) << 7) | self.payload_type;
        header[2..4].copy_from_slice(&self.sequence_number.to_be_bytes());
        header[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        header[8..12].copy_from_slice(&self.ssrc.to_be_bytes());

        let mut packet = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        packet.extend_from_slice(&header);
        packet.extend_from_slice(&self.payload);

        packet
    }

    /// Decodes a byte stream into an RtpPacket.
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(anyhow!(
                "RTP packet too small: {} bytes",
                data.len()
            ));
        }

        let header = &data[..HEADER_SIZE];
        let payload = Bytes::copy_from_slice(&data[HEADER_SIZE..]);

        let version = header[0] >> 6;
        if version != RTP_VERSION {
            return Err(anyhow!("Invalid RTP version: {}", version));
        }

        let padding = (header[0] >> 5) & 1 == 1;
        let extension = (header[0] >> 4) & 1 == 1;
        let cc = header[0] & 0x0F;
        let marker = (header[1] >> 7) & 1 == 1;
        let payload_type = header[1] & 0x7F;
        let sequence_number = u16::from_be_bytes(header[2..4].try_into()?);
        let timestamp = u32::from_be_bytes(header[4..8].try_into()?);
        let ssrc = u32::from_be_bytes(header[8..12].try_into()?);

        Ok(Self {
            version,
            padding,
            extension,
            cc,
            marker,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            payload,
        })
    }
}