use serde::Serialize;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    time::Duration,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RpcError {
    Transport(String),
    InvalidResponse(String),
    Remote(Value),
}

pub trait Rpc {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError>;
}

#[derive(Clone, Debug, Serialize)]
pub struct RpcEvidence {
    pub request: Value,
    pub response: Option<Value>,
    pub result: Result<Value, RpcError>,
}

/// HTTP client for the driver's private Anvil loopback socket.
pub struct AnvilRpc {
    addr: SocketAddr,
    next_id: u64,
    timeout: Duration,
    pub evidence: Vec<RpcEvidence>,
}

impl AnvilRpc {
    pub fn new(addr: SocketAddr) -> std::io::Result<Self> {
        if !addr.ip().is_loopback() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Anvil endpoint must be loopback",
            ));
        }
        Ok(Self {
            addr,
            next_id: 0,
            timeout: Duration::from_secs(5),
            evidence: Vec::new(),
        })
    }

    fn exchange(
        &self,
        request: &Value,
        wire_response: &mut Option<Value>,
    ) -> Result<Value, RpcError> {
        let transport = |e: std::io::Error| RpcError::Transport(e.to_string());
        let invalid = |s: &str| RpcError::InvalidResponse(s.into());
        let body = request.to_string();
        let mut stream = TcpStream::connect_timeout(&self.addr, self.timeout).map_err(transport)?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(transport)?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(transport)?;
        write!(
            stream,
            "POST / HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
            self.addr, body.len(), body
        )
        .map_err(transport)?;
        const LIMIT: u64 = 32 * 1024 * 1024;
        let mut response = Vec::new();
        stream
            .take(LIMIT + 1)
            .read_to_end(&mut response)
            .map_err(transport)?;
        if response.len() as u64 > LIMIT {
            return Err(invalid("HTTP response exceeded size limit"));
        }
        let boundary = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or_else(|| invalid("incomplete HTTP response"))?;
        let header =
            std::str::from_utf8(&response[..boundary]).map_err(|_| invalid("HTTP header"))?;
        let body = &response[boundary + 4..];
        let mut headers = header.split("\r\n");
        if headers
            .next()
            .and_then(|status| status.split_ascii_whitespace().nth(1))
            != Some("200")
        {
            return Err(invalid("HTTP status was not 200"));
        }
        let mut length = None;
        for line in headers {
            if let Some((name, value)) = line.split_once(':') {
                if name.eq_ignore_ascii_case("content-length") {
                    if length.is_some() {
                        return Err(invalid("duplicate HTTP Content-Length"));
                    }
                    length = Some(
                        value
                            .trim()
                            .parse::<usize>()
                            .map_err(|_| invalid("HTTP Content-Length"))?,
                    );
                }
                if name.eq_ignore_ascii_case("transfer-encoding") {
                    return Err(invalid("unexpected transfer encoding from pinned Anvil"));
                }
            }
        }
        if length.is_some_and(|length| length != body.len()) {
            return Err(invalid("HTTP body length mismatch"));
        }
        let response: Value = serde_json::from_slice(body).map_err(|e| invalid(&e.to_string()))?;
        *wire_response = Some(response.clone());
        if response["jsonrpc"] != "2.0" || response["id"] != request["id"] {
            return Err(invalid("unexpected JSON-RPC version or id"));
        }
        match (response.get("result"), response.get("error")) {
            (Some(result), None) => Ok(result.clone()),
            (None, Some(error)) if error.is_object() && error["code"].is_i64() => {
                Err(RpcError::Remote(error.clone()))
            }
            _ => Err(invalid("JSON-RPC response requires one result or error")),
        }
    }
}

impl Rpc for AnvilRpc {
    fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcError> {
        let request =
            json!({"jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params});
        self.next_id = self.next_id.checked_add(1).expect("JSON-RPC id exhausted");
        let mut response = None;
        let result = self.exchange(&request, &mut response);
        self.evidence.push(RpcEvidence {
            request,
            response,
            result: result.clone(),
        });
        result
    }
}
