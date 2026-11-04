use serde_json::Value;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::{Duration, Instant};

const OLLAMA_ADDRESS: &str = "127.0.0.1:11434";
const MAX_PROMPT_BYTES: usize = 4096;
const MAX_RESPONSE_BYTES: usize = 16 * 1024;
const MAX_HTTP_BYTES: usize = 256 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, PartialEq, Eq)]
pub enum ProviderError {
    InvalidPrompt,
    RequestTooLarge,
    InvalidFrame,
    Cancelled,
    Timeout,
    Ollama(String),
    Protocol(String),
    Io(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPrompt => write!(
                formatter,
                "prompt must contain non-whitespace text without NUL bytes"
            ),
            Self::RequestTooLarge => write!(formatter, "prompt exceeds the 4096-byte limit"),
            Self::InvalidFrame => write!(formatter, "invalid provider request frame"),
            Self::Cancelled => write!(formatter, "request cancelled"),
            Self::Timeout => write!(formatter, "local model request timed out"),
            Self::Ollama(message) | Self::Protocol(message) | Self::Io(message) => {
                write!(formatter, "{message}")
            }
        }
    }
}

impl std::error::Error for ProviderError {}

pub fn validate_prompt(bytes: &[u8]) -> Result<&str, ProviderError> {
    if bytes.len() > MAX_PROMPT_BYTES {
        return Err(ProviderError::RequestTooLarge);
    }
    let prompt = std::str::from_utf8(bytes).map_err(|_| ProviderError::InvalidPrompt)?;
    if prompt.trim().is_empty() || prompt.as_bytes().contains(&0) {
        return Err(ProviderError::InvalidPrompt);
    }
    Ok(prompt)
}

fn read_exact_deadline(
    stream: &mut UnixStream,
    bytes: &mut [u8],
    deadline: Instant,
) -> Result<(), ProviderError> {
    let mut received = 0;
    while received < bytes.len() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(ProviderError::Timeout);
        }
        stream
            .set_read_timeout(Some(remaining.min(Duration::from_millis(250))))
            .map_err(io_error)?;
        match stream.read(&mut bytes[received..]) {
            Ok(0) => return Err(ProviderError::InvalidFrame),
            Ok(count) => received += count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                if Instant::now() >= deadline {
                    return Err(ProviderError::Timeout);
                }
            }
            Err(error) => return Err(io_error(error)),
        }
    }
    Ok(())
}

fn read_request(stream: &mut UnixStream) -> Result<String, ProviderError> {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut header = [0; 4];
    read_exact_deadline(stream, &mut header, deadline)?;
    let length = u32::from_be_bytes(header) as usize;
    if length == 0 {
        return Err(ProviderError::InvalidPrompt);
    }
    if length > MAX_PROMPT_BYTES {
        return Err(ProviderError::RequestTooLarge);
    }
    let mut prompt = vec![0; length];
    read_exact_deadline(stream, &mut prompt, deadline)?;
    Ok(validate_prompt(&prompt)?.to_owned())
}

fn client_is_connected(stream: &mut UnixStream) -> Result<(), ProviderError> {
    let mut byte = [0; 1];
    match stream.read(&mut byte) {
        Ok(0) => Err(ProviderError::Cancelled),
        Ok(_) => Err(ProviderError::InvalidFrame),
        Err(error)
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
            ) =>
        {
            Ok(())
        }
        Err(error) => Err(io_error(error)),
    }
}

fn ollama_request(
    path: &str,
    body: Option<&[u8]>,
    client: &mut UnixStream,
) -> Result<Vec<u8>, ProviderError> {
    use std::net::{SocketAddr, TcpStream};

    let address: SocketAddr = OLLAMA_ADDRESS.parse().expect("fixed loopback address");
    let deadline = Instant::now() + REQUEST_TIMEOUT;
    let mut upstream = TcpStream::connect_timeout(&address, Duration::from_secs(2))
        .map_err(|error| ProviderError::Ollama(format!("cannot reach local Ollama: {error}")))?;
    client
        .set_read_timeout(Some(Duration::from_millis(250)))
        .map_err(io_error)?;
    upstream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .map_err(io_error)?;
    upstream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .map_err(io_error)?;

    let mut request = format!(
        "{} {path} HTTP/1.1\r\nHost: 127.0.0.1:11434\r\nConnection: close\r\n",
        if body.is_some() { "POST" } else { "GET" }
    )
    .into_bytes();
    if let Some(body) = body {
        request.extend_from_slice(b"Content-Type: application/json\r\nContent-Length: ");
        request.extend_from_slice(body.len().to_string().as_bytes());
        request.extend_from_slice(b"\r\n");
    }
    request.extend_from_slice(b"\r\n");
    if let Some(body) = body {
        request.extend_from_slice(body);
    }
    upstream.write_all(&request).map_err(io_error)?;

    let mut response = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        client_is_connected(client)?;
        if Instant::now() >= deadline {
            return Err(ProviderError::Timeout);
        }
        match upstream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                if response.len() + count > MAX_HTTP_BYTES {
                    return Err(ProviderError::Protocol(
                        "Ollama response exceeds the 256-KiB limit".to_owned(),
                    ));
                }
                response.extend_from_slice(&buffer[..count]);
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(ProviderError::Ollama(error.to_string())),
        }
    }
    let (status, body) = parse_http_response(&response)?;
    if status != 200 {
        let detail = ollama_json(&body)
            .ok()
            .and_then(|value| {
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| format!("Ollama returned HTTP {status}"));
        return Err(ProviderError::Ollama(detail));
    }
    Ok(body)
}

fn parse_http_response(response: &[u8]) -> Result<(u16, Vec<u8>), ProviderError> {
    let split = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| ProviderError::Protocol("malformed Ollama HTTP response".to_owned()))?;
    let headers = std::str::from_utf8(&response[..split])
        .map_err(|_| ProviderError::Protocol("invalid Ollama HTTP headers".to_owned()))?;
    let status = headers
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| ProviderError::Protocol("invalid Ollama HTTP status".to_owned()))?;
    let body = &response[split + 4..];
    let chunked = headers.lines().any(|line| {
        line.split_once(':').is_some_and(|(name, value)| {
            name.eq_ignore_ascii_case("transfer-encoding")
                && value.trim().eq_ignore_ascii_case("chunked")
        })
    });
    let body = if chunked {
        decode_chunked(body)?
    } else {
        body.to_vec()
    };
    Ok((status, body))
}

fn decode_chunked(mut body: &[u8]) -> Result<Vec<u8>, ProviderError> {
    let mut decoded = Vec::new();
    loop {
        let line_end = body
            .windows(2)
            .position(|window| window == b"\r\n")
            .ok_or_else(|| {
                ProviderError::Protocol("malformed chunked Ollama response".to_owned())
            })?;
        let length_text = std::str::from_utf8(&body[..line_end])
            .map_err(|_| ProviderError::Protocol("invalid chunk length".to_owned()))?;
        let length = usize::from_str_radix(length_text.split(';').next().unwrap_or(""), 16)
            .map_err(|_| ProviderError::Protocol("invalid chunk length".to_owned()))?;
        body = &body[line_end + 2..];
        if length == 0 {
            return Ok(decoded);
        }
        let end = length
            .checked_add(2)
            .filter(|end| *end <= body.len())
            .ok_or_else(|| {
                ProviderError::Protocol("truncated chunked Ollama response".to_owned())
            })?;
        if body[end - 2..end] != *b"\r\n" || decoded.len() + length > MAX_HTTP_BYTES {
            return Err(ProviderError::Protocol(
                "invalid chunked Ollama response".to_owned(),
            ));
        }
        decoded.extend_from_slice(&body[..length]);
        body = &body[end..];
    }
}

fn ollama_json(body: &[u8]) -> Result<Value, ProviderError> {
    serde_json::from_slice(body)
        .map_err(|error| ProviderError::Protocol(format!("invalid Ollama JSON: {error}")))
}

fn verify_model(model: &str) -> Result<(), ProviderError> {
    let (status, body) = {
        let address: std::net::SocketAddr = OLLAMA_ADDRESS.parse().expect("fixed loopback address");
        let mut upstream = std::net::TcpStream::connect_timeout(&address, Duration::from_secs(2))
            .map_err(|error| {
            ProviderError::Ollama(format!("cannot reach local Ollama: {error}"))
        })?;
        upstream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(io_error)?;
        upstream
            .write_all(
                b"GET /api/tags HTTP/1.1\r\nHost: 127.0.0.1:11434\r\nConnection: close\r\n\r\n",
            )
            .map_err(io_error)?;
        let mut response = Vec::new();
        upstream
            .take((MAX_HTTP_BYTES + 1) as u64)
            .read_to_end(&mut response)
            .map_err(io_error)?;
        if response.len() > MAX_HTTP_BYTES {
            return Err(ProviderError::Protocol(
                "Ollama response exceeds the 256-KiB limit".to_owned(),
            ));
        }
        parse_http_response(&response)?
    };
    if status != 200 {
        return Err(ProviderError::Ollama(format!(
            "Ollama model inventory returned HTTP {status}"
        )));
    }
    let inventory = ollama_json(&body)?;
    let present = inventory
        .get("models")
        .and_then(Value::as_array)
        .is_some_and(|models| {
            models
                .iter()
                .any(|entry| entry.get("name").and_then(Value::as_str) == Some(model))
        });
    if present {
        Ok(())
    } else {
        Err(ProviderError::Ollama(format!(
            "required local model is not installed: {model}"
        )))
    }
}

pub fn serve(socket_path: &Path, model: &str) -> Result<(), ProviderError> {
    if !socket_path.is_absolute() || model.trim().is_empty() {
        return Err(ProviderError::Protocol(
            "socket path must be absolute and model name nonempty".to_owned(),
        ));
    }
    verify_model(model)?;

    let parent = socket_path
        .parent()
        .ok_or_else(|| ProviderError::Protocol("socket path has no parent directory".to_owned()))?;
    let directory = std::fs::symlink_metadata(parent).map_err(io_error)?;
    if !directory.file_type().is_dir() || directory.mode() & 0o077 != 0 {
        return Err(ProviderError::Protocol(
            "socket directory must be a private, non-symlink directory".to_owned(),
        ));
    }
    let listener = UnixListener::bind(socket_path).map_err(io_error)?;
    std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o600))
        .map_err(io_error)?;
    println!(
        "LOCAL_MODEL_READY: socket={} model={model}",
        socket_path.display()
    );

    for connection in listener.incoming() {
        match connection {
            Ok(mut stream) => {
                let request = read_request(&mut stream);
                let response = request.and_then(|prompt| generate(&prompt, model, &mut stream));
                if let Err(error) = write_response(&mut stream, response) {
                    eprintln!("LOCAL_MODEL_CLIENT_ERROR: {error}");
                }
            }
            Err(error) => eprintln!("LOCAL_MODEL_ACCEPT_ERROR: {error}"),
        }
    }
    drop(listener);
    std::fs::remove_file(socket_path).map_err(io_error)?;
    Ok(())
}

fn generate(prompt: &str, model: &str, client: &mut UnixStream) -> Result<String, ProviderError> {
    let body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "think": false,
        "options": { "num_predict": 256 }
    });
    let body =
        serde_json::to_vec(&body).map_err(|error| ProviderError::Protocol(error.to_string()))?;
    let response = ollama_request("/api/generate", Some(&body), client)?;
    let value = ollama_json(&response)?;
    let text = value
        .get("response")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ProviderError::Ollama(
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("Ollama response did not contain generated text")
                    .to_owned(),
            )
        })?;
    if text.trim().is_empty() {
        return Err(ProviderError::Ollama(
            "Ollama returned an empty response".to_owned(),
        ));
    }
    if text.len() > MAX_RESPONSE_BYTES {
        return Err(ProviderError::Protocol(
            "generated response exceeds the 16-KiB limit".to_owned(),
        ));
    }
    Ok(text.to_owned())
}

fn write_response(
    stream: &mut UnixStream,
    response: Result<String, ProviderError>,
) -> io::Result<()> {
    let (status, text) = match response {
        Ok(text) => (0_u8, text),
        Err(error) => (1_u8, error.to_string()),
    };
    let bytes = text.as_bytes();
    let length = bytes.len().min(MAX_RESPONSE_BYTES) as u32;
    stream.write_all(&[status])?;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(&bytes[..length as usize])
}

fn io_error(error: io::Error) -> ProviderError {
    ProviderError::Io(error.to_string())
}

use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[cfg(test)]
mod tests {
    use super::{
        MAX_PROMPT_BYTES, ProviderError, decode_chunked, ollama_json, parse_http_response,
        read_exact_deadline, read_request, validate_prompt,
    };
    use std::io::Write;
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;

    #[test]
    fn accepts_utf8_prompts_within_the_limit() {
        assert_eq!(
            validate_prompt("Summarize this".as_bytes()),
            Ok("Summarize this")
        );
        assert!(validate_prompt("é".repeat(MAX_PROMPT_BYTES / 2).as_bytes()).is_ok());
    }

    #[test]
    fn rejects_empty_invalid_utf8_nul_and_oversized_prompts() {
        assert_eq!(validate_prompt(b" \t\n"), Err(ProviderError::InvalidPrompt));
        assert_eq!(validate_prompt(&[0xff]), Err(ProviderError::InvalidPrompt));
        assert_eq!(validate_prompt(b"a\0b"), Err(ProviderError::InvalidPrompt));
        assert_eq!(
            validate_prompt(&vec![b'a'; MAX_PROMPT_BYTES + 1]),
            Err(ProviderError::RequestTooLarge)
        );
    }

    #[test]
    fn rejects_zero_oversized_and_truncated_request_frames() {
        let (mut client, mut server) = UnixStream::pair().unwrap();
        client.write_all(&0_u32.to_be_bytes()).unwrap();
        assert_eq!(read_request(&mut server), Err(ProviderError::InvalidPrompt));

        let (mut client, mut server) = UnixStream::pair().unwrap();
        client
            .write_all(&((MAX_PROMPT_BYTES as u32) + 1).to_be_bytes())
            .unwrap();
        assert_eq!(
            read_request(&mut server),
            Err(ProviderError::RequestTooLarge)
        );

        let (mut client, mut server) = UnixStream::pair().unwrap();
        client.write_all(&3_u32.to_be_bytes()).unwrap();
        client.write_all(b"ab").unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        assert_eq!(read_request(&mut server), Err(ProviderError::InvalidFrame));
    }

    #[test]
    fn times_out_before_reading_after_the_frame_deadline() {
        let (_client, mut server) = UnixStream::pair().unwrap();
        assert_eq!(
            read_exact_deadline(&mut server, &mut [0], std::time::Instant::now()),
            Err(ProviderError::Timeout)
        );
    }

    #[test]
    fn parses_bounded_http_headers_and_chunked_bodies() {
        let response =
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\ntest\r\n0\r\n\r\n";
        let (status, body) = parse_http_response(response).unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, b"test");
        assert!(decode_chunked(b"3\r\nno").is_err());
    }

    #[test]
    fn rejects_malformed_http_and_json_responses() {
        assert!(parse_http_response(b"not HTTP").is_err());
        assert!(ollama_json(b"{bad json").is_err());
    }
}
