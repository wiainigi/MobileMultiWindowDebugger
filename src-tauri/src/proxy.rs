//! 每窗口独立的本地反向代理。
//!
//! iframe 的存储（Cookie / localStorage / IndexedDB）是按 origin 划分的，
//! 若直接加载目标站点，同一站点的所有窗口会共享存储。
//! 这里为每个无痕窗口在 `127.0.0.1` 上分配一个**独立端口**并转发到目标站点，
//! 使每个无痕窗口的 origin 都是独一无二的 `http://127.0.0.1:<port>`，
//! 从而获得真正互相隔离的存储空间，而页面本身（含 WebSocket HMR）仍可正常运作。

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::header::HeaderMap;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

type ProxyBody = Full<Bytes>;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// 逐跳首部不允许被代理转发
const HOP_BY_HOP: [&str; 9] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

/// 允许 iframe 嵌入：去掉站点自身的嵌入限制，交由本工具统一控制
const STRIPPED_RESPONSE_HEADERS: [&str; 3] = [
    "x-frame-options",
    "content-security-policy",
    "content-security-policy-report-only",
];

#[derive(Default)]
pub struct ProxyState {
    shutdowns: Mutex<HashMap<u16, oneshot::Sender<()>>>,
}

impl ProxyState {
    fn register(&self, port: u16, sender: oneshot::Sender<()>) {
        if let Ok(mut map) = self.shutdowns.lock() {
            map.insert(port, sender);
        }
    }

    pub fn stop(&self, port: u16) {
        if let Ok(mut map) = self.shutdowns.lock() {
            if let Some(sender) = map.remove(&port) {
                let _ = sender.send(());
            }
        }
    }
}

#[derive(Clone)]
struct Target {
    scheme: String,
    /// host[:port]，用于 Host 首部
    authority: String,
    host: String,
    port: u16,
}

struct Ctx {
    /// 本地代理端口，页面 origin 即 `http://127.0.0.1:{port}`
    port: u16,
    target: Target,
    client: reqwest::Client,
}

fn parse_target(origin: &str) -> Result<Target, String> {
    let parsed = url::Url::parse(origin).map_err(|error| format!("无法解析地址 {origin}: {error}"))?;
    let scheme = parsed.scheme().to_string();
    if scheme != "http" && scheme != "https" {
        return Err(format!("仅支持 http/https，当前为 {scheme}"));
    }
    let host = parsed.host_str().ok_or_else(|| "地址缺少主机名".to_string())?.to_string();
    let port = parsed.port_or_known_default().ok_or_else(|| "地址缺少端口".to_string())?;
    let authority = match parsed.port() {
        Some(explicit) => format!("{host}:{explicit}"),
        None => host.clone(),
    };
    Ok(Target {
        scheme,
        authority,
        host,
        port,
    })
}

/// 启动一个绑定到随机端口的代理，返回本地端口号
pub async fn start(origin: String, state: &ProxyState) -> Result<u16, String> {
    let target = parse_target(&origin)?;

    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|error| format!("无法监听本地端口: {error}"))?;
    let port = listener.local_addr().map_err(|error| error.to_string())?.port();

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|error| error.to_string())?;

    let ctx = Arc::new(Ctx { port, target, client });
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    state.register(port, shutdown_tx);

    println!("[proxy] 127.0.0.1:{port} -> {}://{}", ctx.target.scheme, ctx.target.authority);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accepted = listener.accept() => {
                    let Ok((stream, _)) = accepted else { continue };
                    let io = TokioIo::new(stream);
                    let ctx = ctx.clone();
                    tokio::spawn(async move {
                        let service = service_fn(move |req| {
                            let ctx = ctx.clone();
                            async move { handle(req, ctx).await }
                        });
                        let _ = hyper::server::conn::http1::Builder::new()
                            .serve_connection(io, service)
                            .with_upgrades()
                            .await;
                    });
                }
            }
        }
        println!("[proxy] 127.0.0.1:{port} 已停止");
    });

    Ok(port)
}

async fn handle(req: Request<Incoming>, ctx: Arc<Ctx>) -> Result<Response<ProxyBody>, Infallible> {
    let response = if is_websocket_upgrade(req.headers()) {
        tunnel_websocket(req, ctx).await
    } else {
        forward(req, ctx).await
    };
    Ok(response)
}

fn is_websocket_upgrade(headers: &HeaderMap) -> bool {
    headers
        .get("upgrade")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("websocket"))
}

fn error_response(status: StatusCode, message: &str) -> Response<ProxyBody> {
    println!("[proxy] {status}: {message}");
    let body = format!(
        "<html><body style=\"font-family:system-ui;background:#0b0f16;color:#e6edf7;padding:24px\"><h3>代理错误</h3><p>{message}</p></body></html>"
    );
    Response::builder()
        .status(status)
        .header("content-type", "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(body)))
        .unwrap_or_else(|_| Response::new(Full::new(Bytes::new())))
}

fn path_and_query(req: &Request<Incoming>) -> String {
    req.uri()
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| "/".to_string())
}

async fn forward(req: Request<Incoming>, ctx: Arc<Ctx>) -> Response<ProxyBody> {
    let (parts, body) = req.into_parts();
    let path = path_and_query_from_parts(&parts.uri);
    let upstream_url = format!("{}://{}{}", ctx.target.scheme, ctx.target.authority, path);

    let request_body = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(error) => return error_response(StatusCode::BAD_REQUEST, &format!("读取请求体失败: {error}")),
    };

    let upstream_origin = format!("{}://{}", ctx.target.scheme, ctx.target.authority);
    let mut headers = HeaderMap::new();
    for (name, value) in parts.headers.iter() {
        let key = name.as_str().to_ascii_lowercase();
        // 逐跳首部丢弃；Host/Origin/Referer 需要改写为上游视角
        if HOP_BY_HOP.contains(&key.as_str()) || key == "host" || key == "content-length" {
            continue;
        }
        if key == "origin" {
            continue;
        }
        if key == "referer" {
            headers.insert(name.clone(), header_value(&format!("{upstream_origin}/")));
            continue;
        }
        headers.insert(name.clone(), value.clone());
    }
    headers.insert("host", header_value(&ctx.target.authority));
    headers.insert("origin", header_value(&upstream_origin));

    let upstream = match ctx
        .client
        .request(parts.method.clone(), &upstream_url)
        .headers(headers)
        .body(request_body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            return error_response(
                StatusCode::BAD_GATEWAY,
                &format!("无法访问 {}:{}\n\n{error}", ctx.target.host, ctx.target.port),
            )
        }
    };

    let status = upstream.status();
    let response_headers = upstream.headers().clone();
    let response_body = upstream.bytes().await.unwrap_or_default();

    let mut builder = Response::builder().status(status.as_u16());
    for (name, value) in response_headers.iter() {
        let key = name.as_str().to_ascii_lowercase();
        if HOP_BY_HOP.contains(&key.as_str()) || key == "content-length" {
            continue;
        }
        if STRIPPED_RESPONSE_HEADERS.contains(&key.as_str()) {
            continue;
        }
        match key.as_str() {
            // 重定向指回本地代理端口，保持窗口内的 origin 不变
            "location" => {
                let rewritten = rewrite_location(value.to_str().unwrap_or_default(), &ctx);
                builder = builder.header(name.clone(), header_value(&rewritten));
            }
            // Cookie 归属本地代理 origin，去掉 Domain/Secure 以便在 127.0.0.1 上生效
            "set-cookie" => {
                let rewritten = rewrite_cookie(value.to_str().unwrap_or_default());
                builder = builder.header(name.clone(), header_value(&rewritten));
            }
            _ => {
                builder = builder.header(name.clone(), value.clone());
            }
        }
    }

    builder
        .body(Full::new(response_body))
        .unwrap_or_else(|_| error_response(StatusCode::INTERNAL_SERVER_ERROR, "构造响应失败"))
}

fn path_and_query_from_parts(uri: &hyper::Uri) -> String {
    uri.path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| "/".to_string())
}

fn header_value(raw: &str) -> hyper::header::HeaderValue {
    hyper::header::HeaderValue::from_str(raw).unwrap_or_else(|_| hyper::header::HeaderValue::from_static(""))
}

/// 把指向上游的重定向改写为本地代理地址
fn rewrite_location(location: &str, ctx: &Ctx) -> String {
    let Ok(parsed) = url::Url::parse(location) else {
        return location.to_string();
    };
    let same_host = parsed.host_str() == Some(ctx.target.host.as_str())
        && parsed.port_or_known_default() == Some(ctx.target.port);
    if !same_host {
        return location.to_string();
    }
    let query = parsed.query().map(|value| format!("?{value}")).unwrap_or_default();
    let fragment = parsed.fragment().map(|value| format!("#{value}")).unwrap_or_default();
    format!("http://127.0.0.1:{}{}{}{}", ctx.port, parsed.path(), query, fragment)
}

/// 去掉 Domain 与 Secure，让 Cookie 归属于本地代理 origin
fn rewrite_cookie(raw: &str) -> String {
    raw.split(';')
        .map(str::trim)
        .filter(|part| {
            let lower = part.to_ascii_lowercase();
            !lower.starts_with("domain=") && lower != "secure" && !lower.starts_with("partitioned")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// WebSocket（如 Vite HMR）隧道：仅支持 http 上游，握手后做裸 TCP 双向转发
async fn tunnel_websocket(mut req: Request<Incoming>, ctx: Arc<Ctx>) -> Response<ProxyBody> {
    if ctx.target.scheme != "http" {
        return error_response(StatusCode::BAD_GATEWAY, "暂不支持 https 站点的 WebSocket");
    }

    let path = path_and_query(&req);
    let on_upgrade = hyper::upgrade::on(&mut req);

    let mut upstream = match TcpStream::connect((ctx.target.host.as_str(), ctx.target.port)).await {
        Ok(stream) => stream,
        Err(error) => return error_response(StatusCode::BAD_GATEWAY, &format!("WebSocket 连接失败: {error}")),
    };

    let mut handshake = format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n",
        ctx.target.authority
    );
    for key in [
        "sec-websocket-key",
        "sec-websocket-version",
        "sec-websocket-protocol",
        "sec-websocket-extensions",
        "cookie",
        "origin",
        "user-agent",
    ] {
        if let Some(value) = req.headers().get(key).and_then(|value| value.to_str().ok()) {
            handshake.push_str(&format!("{key}: {value}\r\n"));
        }
    }
    handshake.push_str("\r\n");

    if upstream.write_all(handshake.as_bytes()).await.is_err() {
        return error_response(StatusCode::BAD_GATEWAY, "WebSocket 握手发送失败");
    }

    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 2048];
    let header_end = loop {
        match upstream.read(&mut chunk).await {
            Ok(0) => break None,
            Ok(read) => {
                buffer.extend_from_slice(&chunk[..read]);
                if let Some(index) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
                    break Some(index);
                }
                if buffer.len() > 64 * 1024 {
                    break None;
                }
            }
            Err(_) => break None,
        }
    };

    let Some(header_end) = header_end else {
        return error_response(StatusCode::BAD_GATEWAY, "WebSocket 握手响应无效");
    };

    let head = String::from_utf8_lossy(&buffer[..header_end]).to_string();
    let mut lines = head.split("\r\n");
    let status_code: u16 = lines
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    if status_code != 101 {
        return error_response(StatusCode::BAD_GATEWAY, "上游未升级为 WebSocket");
    }

    let mut accept = String::new();
    let mut protocol = String::new();
    for line in lines {
        if let Some((key, value)) = line.split_once(':') {
            match key.trim().to_ascii_lowercase().as_str() {
                "sec-websocket-accept" => accept = value.trim().to_string(),
                "sec-websocket-protocol" => protocol = value.trim().to_string(),
                _ => {}
            }
        }
    }

    let rest = buffer[header_end + 4..].to_vec();

    tokio::spawn(async move {
        let Ok(upgraded) = on_upgrade.await else { return };
        let mut client_io = TokioIo::new(upgraded);
        if !rest.is_empty() && client_io.write_all(&rest).await.is_err() {
            return;
        }
        let _ = tokio::io::copy_bidirectional(&mut client_io, &mut upstream).await;
    });

    let mut builder = Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header("upgrade", "websocket")
        .header("connection", "Upgrade");
    if !accept.is_empty() {
        builder = builder.header("sec-websocket-accept", accept);
    }
    if !protocol.is_empty() {
        builder = builder.header("sec-websocket-protocol", protocol);
    }
    builder
        .body(Full::new(Bytes::new()))
        .unwrap_or_else(|_| error_response(StatusCode::BAD_GATEWAY, "构造升级响应失败"))
}
