//! Defines RTSP request/response structures and parsing logic.

use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

pub const RTSP_VERSION: &str = "RTSP/1.0";

/// Represents the type of an RTSP request.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RequestType {
    Setup,
    Play,
    Pause,
    Teardown,
}

impl RequestType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestType::Setup => "SETUP",
            RequestType::Play => "PLAY",
            RequestType::Pause => "PAUSE",
            RequestType::Teardown => "TEARDOWN",
        }
    }
}

/// Represents a parsed RTSP request from a client.
#[derive(Debug)]
pub struct RtspRequest {
    pub request_type: RequestType,
    pub filename: String,
    pub cseq: u32,
    pub transport: Option<String>,
    pub client_port: Option<u16>,
    pub session_id: Option<u32>,
}

impl RtspRequest {
    /// Parses a raw RTSP request string into an `RtspRequest` struct.
    pub fn parse(data: &str) -> Result<Self> {
        let mut lines = data.lines();

        // Parse request line
        let request_line = lines.next().ok_or_else(|| anyhow!("Empty request"))?;
        let mut parts = request_line.split_whitespace();
        let method_str = parts.next().ok_or_else(|| anyhow!("Missing method"))?;
        let filename = parts
            .next()
            .ok_or_else(|| anyhow!("Missing filename/URI"))?
            .to_string();
        let _version = parts.next().ok_or_else(|| anyhow!("Missing version"))?;

        let request_type = match method_str {
            "SETUP" => RequestType::Setup,
            "PLAY" => RequestType::Play,
            "PAUSE" => RequestType::Pause,
            "TEARDOWN" => RequestType::Teardown,
            _ => bail!("Unsupported RTSP method: {}", method_str),
        };

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        let cseq = headers
            .get("cseq")
            .ok_or_else(|| anyhow!("Missing CSeq header"))?
            .parse()?;

        let session_id = headers.get("session").map(|s| s.parse()).transpose()?;

        let (transport, client_port) = if let Some(transport_str) = headers.get("transport") {
            let client_port = transport_str
                .split(';')
                .find_map(|part| part.trim().strip_prefix("client_port="))
                .map(|p| p.trim().parse())
                .transpose()?;
            (Some(transport_str.to_string()), client_port)
        } else {
            (None, None)
        };

        Ok(Self {
            request_type,
            filename,
            cseq,
            transport,
            client_port,
            session_id,
        })
    }
}

/// Represents a parsed RTSP response from the server.
#[derive(Debug)]
pub struct RtspResponse {
    pub status_code: u16,
    pub cseq: u32,
    pub session_id: u32,
}

impl RtspResponse {
    /// Parses a raw RTSP response string.
    pub fn parse(data: &str) -> Result<Self> {
        let mut lines = data.lines();

        // Parse status line
        let status_line = lines.next().ok_or_else(|| anyhow!("Empty response"))?;
        let mut parts = status_line.split_whitespace();
        let _version = parts.next();
        let status_code_str = parts.next().ok_or_else(|| anyhow!("Missing status code"))?;
        let status_code = status_code_str.parse()?;

        // Parse headers
        let mut headers = HashMap::new();
        for line in lines {
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        let cseq = headers
            .get("cseq")
            .ok_or_else(|| anyhow!("Missing CSeq header"))?
            .parse()?;
        let session_id = headers
            .get("session")
            .ok_or_else(|| anyhow!("Missing Session header"))?
            .parse()?;

        Ok(Self {
            status_code,
            cseq,
            session_id,
        })
    }
}