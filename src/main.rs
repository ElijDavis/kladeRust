use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

const DEFAULT_API: &str = "https://jsonplaceholder.typicode.com/todos/1";

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8080").expect("bind 0.0.0.0:8080");
    println!("Klade Rust probe listening on 0.0.0.0:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 2048];
                let _ = stream.read(&mut buffer);
                let request = String::from_utf8_lossy(&buffer);
                let path = request.split_whitespace().nth(1).unwrap_or("/");
                let payload = probe_payload();
                let (status, content_type, body) = if path.starts_with("/api") {
                    (
                        "200 OK",
                        "application/json",
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".into()),
                    )
                } else {
                    ("200 OK", "text/html; charset=utf-8", render_html(&payload))
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
            }
            Err(error) => eprintln!("Connection failed: {error}"),
        }
    }
}

fn probe_payload() -> serde_json::Value {
    let app_name = env::var("APP_NAME").unwrap_or_default();
    let api_url = env::var("API_URL").unwrap_or_else(|_| DEFAULT_API.to_string());
    let remote = fetch_remote(&api_url);
    serde_json::json!({
        "framework": "rust",
        "appName": if app_name.is_empty() { "(not set)" } else { &app_name },
        "appNameSet": !app_name.is_empty(),
        "apiUrl": api_url,
        "apiUrlFromEnv": env::var("API_URL").is_ok(),
        "remote": remote,
    })
}

fn fetch_remote(api_url: &str) -> serde_json::Value {
    match ureq::get(api_url)
        .timeout(Duration::from_secs(8))
        .call()
        .and_then(|response| response.into_json::<serde_json::Value>())
    {
        Ok(value) => value,
        Err(error) => serde_json::json!({ "error": error.to_string() }),
    }
}

fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn render_html(payload: &serde_json::Value) -> String {
    let injected = payload["appNameSet"].as_bool().unwrap_or(false);
    let badge = if injected { "ok" } else { "bad" };
    let badge_label = if injected {
        "APP_NAME injected"
    } else {
        "APP_NAME missing"
    };
    let app_name = esc(payload["appName"].as_str().unwrap_or("(not set)"));
    let api_url = esc(payload["apiUrl"].as_str().unwrap_or(DEFAULT_API));
    let remote = esc(&serde_json::to_string_pretty(&payload["remote"]).unwrap_or_default());
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8"/>
  <meta name="viewport" content="width=device-width, initial-scale=1"/>
  <title>Klade Rust Probe</title>
  <style>
    body {{ margin:0; font-family:ui-sans-serif,system-ui,sans-serif; background:#0b1220; color:#e8eef7; }}
    main {{ max-width:44rem; margin:0 auto; padding:2.5rem 1.25rem; }}
    h1 {{ margin:0 0 .4rem; font-size:1.6rem; }}
    .sub {{ color:#9fb0c8; margin-bottom:1.5rem; }}
    .card {{ background:#121b2c; border:1px solid #24324a; border-radius:12px; padding:1.1rem 1.2rem; margin-bottom:1rem; }}
    .row {{ display:flex; justify-content:space-between; gap:1rem; padding:.45rem 0; border-bottom:1px solid #1e2a40; }}
    .row:last-child {{ border-bottom:0; }}
    .k {{ color:#9fb0c8; }}
    .v {{ font-family:ui-monospace,monospace; word-break:break-all; }}
    .ok {{ color:#4ade80; }} .bad {{ color:#f87171; }}
    pre {{ margin:0; white-space:pre-wrap; word-break:break-word; font-size:.85rem; }}
  </style>
</head>
<body>
  <main>
    <h1>Klade Rust probe</h1>
    <p class="sub">Set <code>APP_NAME</code> and <code>API_URL</code> in Klade. This binary reads them at runtime.</p>
    <div class="card">
      <div class="row"><span class="k">Status</span><span class="v {badge}">{badge_label}</span></div>
      <div class="row"><span class="k">APP_NAME</span><span class="v">{app_name}</span></div>
      <div class="row"><span class="k">API_URL</span><span class="v">{api_url}</span></div>
    </div>
    <div class="card">
      <div class="row"><span class="k">Fetched JSON</span><span class="k">GET {api_url}</span></div>
      <pre>{remote}</pre>
    </div>
  </main>
</body>
</html>"#
    )
}
