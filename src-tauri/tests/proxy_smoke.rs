//! 代理冒烟测试：验证转发、重定向改写、Cookie 归属改写与嵌入限制移除。

use std::convert::Infallible;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use mobile_multi_window_debugger_lib::proxy;
use tokio::net::TcpListener;

/// 启动一个最小上游服务，模拟目标站点
async fn spawn_upstream() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            tokio::spawn(async move {
                let io = TokioIo::new(stream);
                let _ = hyper::server::conn::http1::Builder::new()
                    .serve_connection(io, service_fn(handle_upstream))
                    .await;
            });
        }
    });

    port
}

async fn handle_upstream(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path().to_string();
    let response = match path.as_str() {
        "/redirect" => {
            // 模拟站点返回指回自身的绝对跳转地址
            let host = req
                .headers()
                .get("host")
                .and_then(|value| value.to_str().ok())
                .unwrap_or("127.0.0.1")
                .to_string();
            Response::builder()
                .status(302)
                .header("location", format!("http://{host}/landing"))
                .body(Full::new(Bytes::new()))
                .unwrap()
        }
        "/landing" => Response::builder()
            .header("set-cookie", "sid=abc; Domain=127.0.0.1; Path=/; Secure; HttpOnly")
            .header("x-frame-options", "DENY")
            .header("content-security-policy", "frame-ancestors 'none'")
            .body(Full::new(Bytes::from("landing-page")))
            .unwrap(),
        _ => Response::builder()
            .header("content-type", "text/plain")
            .body(Full::new(Bytes::from(format!("echo:{path}"))))
            .unwrap(),
    };
    Ok(response)
}

#[tokio::test]
async fn forwards_request_and_body() {
    let upstream_port = spawn_upstream().await;
    let state = proxy::ProxyState::default();
    let port = proxy::start(format!("http://127.0.0.1:{upstream_port}"), &state).await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{port}/hello?x=1"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "echo:/hello");

    // 代理监听在随机高位端口，说明每个窗口都能拿到独立 origin
    assert!(port > 1024);

    state.stop(port);
}

#[tokio::test]
async fn rewrites_cookie_and_embedding_headers() {
    let upstream_port = spawn_upstream().await;
    let state = proxy::ProxyState::default();
    let port = proxy::start(format!("http://127.0.0.1:{upstream_port}"), &state).await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{port}/landing"))
        .send()
        .await
        .unwrap();

    let cookie = response.headers().get("set-cookie").unwrap().to_str().unwrap().to_lowercase();
    assert!(cookie.contains("sid=abc"), "cookie 名值应保留: {cookie}");
    assert!(!cookie.contains("domain="), "Domain 应被移除: {cookie}");
    assert!(!cookie.contains("secure"), "Secure 应被移除: {cookie}");
    assert!(cookie.contains("httponly"), "其余属性应保留: {cookie}");

    assert!(response.headers().get("x-frame-options").is_none(), "X-Frame-Options 应被移除");
    assert!(
        response.headers().get("content-security-policy").is_none(),
        "CSP 应被移除"
    );

    assert_eq!(response.text().await.unwrap(), "landing-page");

    state.stop(port);
}

#[tokio::test]
async fn rewrites_redirect_to_proxy_origin() {
    let upstream_port = spawn_upstream().await;
    let state = proxy::ProxyState::default();
    let port = proxy::start(format!("http://127.0.0.1:{upstream_port}"), &state).await.unwrap();

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let response = client
        .get(format!("http://127.0.0.1:{port}/redirect"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 302);
    let location = response.headers().get("location").unwrap().to_str().unwrap().to_string();
    assert_eq!(location, format!("http://127.0.0.1:{port}/landing"));

    state.stop(port);
}

#[tokio::test]
async fn stops_accepting_connections_after_shutdown() {
    let upstream_port = spawn_upstream().await;
    let state = proxy::ProxyState::default();
    let port = proxy::start(format!("http://127.0.0.1:{upstream_port}"), &state).await.unwrap();

    let client = reqwest::Client::new();
    assert!(client.get(format!("http://127.0.0.1:{port}/")).send().await.is_ok());

    state.stop(port);
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 换一个 client，避免复用先前建立的 keep-alive 连接
    let fresh_client = reqwest::Client::new();
    assert!(
        fresh_client
            .get(format!("http://127.0.0.1:{port}/"))
            .send()
            .await
            .is_err(),
        "关闭后端口不应再接受连接"
    );
}
