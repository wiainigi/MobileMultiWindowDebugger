pub mod proxy;

use std::time::Duration;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

/// 从 HTML 文本中提取 <title> 内容（大小写不敏感，并折叠空白、还原常用实体）。
fn extract_title(html: &str) -> Option<String> {
    let lowered = html.to_ascii_lowercase();
    let open = lowered.find("<title")?;
    let content_start = lowered[open..].find('>')? + open + 1;
    let content_end = lowered[content_start..].find("</title")? + content_start;

    let decoded = html[content_start..content_end]
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ");

    let collapsed = decoded.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        None
    } else {
        Some(collapsed)
    }
}

/// iframe 受同源策略限制，无法读取跨源页面的 document.title，
/// 因此由 Rust 侧代为请求并解析页面标题。
#[tauri::command]
async fn fetch_page_title(url: String) -> Result<String, String> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("仅支持 http/https 地址".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(12))
        .build()
        .map_err(|error| error.to_string())?;

    let response = client.get(&url).send().await.map_err(|error| {
        println!("[fetch_page_title] 请求失败 {url}: {error}");
        error.to_string()
    })?;

    let status = response.status();
    println!("[fetch_page_title] {url} -> HTTP {}", status.as_u16());

    if !status.is_success() {
        return Err(format!("HTTP {}", status.as_u16()));
    }

    let body = response.text().await.map_err(|error| error.to_string())?;
    let title = extract_title(&body).unwrap_or_default();
    let codepoints = title
        .chars()
        .take(8)
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "[fetch_page_title] {url} -> title={:?} len={} codepoints=[{}] (body {} bytes)",
        title,
        title.chars().count(),
        codepoints,
        body.len()
    );
    Ok(title)
}

/// 开发期诊断用：把前端执行路径输出到 dev 日志
#[tauri::command]
fn debug_log(message: String) {
    println!("[frontend] {message}");
}

/// 为无痕窗口启动独立端口的本地代理，返回本地端口；
/// 该窗口的 origin 由此变为 http://127.0.0.1:{port}，获得完全独立的存储空间。
#[tauri::command]
async fn start_window_proxy(
    origin: String,
    state: tauri::State<'_, proxy::ProxyState>,
) -> Result<u16, String> {
    proxy::start(origin, state.inner()).await
}

/// 关闭窗口时释放对应代理
#[tauri::command]
fn stop_window_proxy(port: u16, state: tauri::State<'_, proxy::ProxyState>) {
    state.stop(port);
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(proxy::ProxyState::default())
        .invoke_handler(tauri::generate_handler![
            greet,
            fetch_page_title,
            debug_log,
            start_window_proxy,
            stop_window_proxy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
