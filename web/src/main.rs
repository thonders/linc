use gurtlib::prelude::*;
use reqwest;
use serde_json::Value;
use tracing::{ info, error };

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = reqwest::Client::new();

    let server = GurtServer::with_tls_certificates("localhost+2.pem", "localhost+2-key.pem")?
        .get("/", |_ctx| async {
            let html = tokio::fs
                ::read_to_string("public/index.html").await
                .unwrap_or_else(|_| "<h1>Error loading page</h1>".to_string());
            Ok(
                GurtResponse::ok()
                    .with_header("content-type", "text/html; charset=utf-8")
                    .with_string_body(html)
            )
        })
        .get("/script.lua", |_ctx| async {
            let script = tokio::fs
                ::read_to_string("public/script.lua").await
                .unwrap_or_else(|_| "-- Error loading script".to_string());
            Ok(
                GurtResponse::ok()
                    .with_header("content-type", "text/plain; charset=utf-8")
                    .with_string_body(script)
            )
        })
        .get("/*", {
            let client = client.clone();
            move |ctx| {
                let client = client.clone();
                let path = ctx.path().to_string();
                async move { handle_redirect(path, client).await }
            }
        });

    info!("GURT server starting on gurt://127.0.0.1:4878");
    server.listen("127.0.0.1:4878").await?;

    Ok(())
}

async fn handle_redirect(path: String, client: reqwest::Client) -> Result<GurtResponse> {
    if path == "/" || path == "/script.lua" {
        return Ok(GurtResponse::not_found().with_string_body("Not found"));
    }

    let short_url = path.trim_start_matches('/');

    info!("Attempting to redirect short URL: {}", short_url);

    let api_url = format!("http://localhost:3333/api/redirect/{}", short_url);

    match client.get(&api_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(json_data) => {
                        if let Some(original_url) = json_data.get("url").and_then(|u| u.as_str()) {
                            info!("Redirecting {} to {}", short_url, original_url);
                            let redirect_html = format!(
                                r#"
                                <!DOCTYPE html>
                                <html>
                                <head>
                                    <title>Redirecting...</title>
                                    <meta http-equiv="refresh" content="0;url={}">
                                </head>
                                <body>
                                    <h1>Redirecting...</h1>
                                    <p>If you are not redirected automatically, <a href="{}">click here</a>.</p>
                                    <p>Original URL: <code>{}</code></p>
                                </body>
                                </html>
                            "#,
                                original_url,
                                original_url,
                                original_url
                            );
                            Ok(
                                GurtResponse::ok()
                                    .with_header("content-type", "text/html; charset=utf-8")
                                    .with_string_body(redirect_html)
                            )
                        } else {
                            error!("Invalid response format from API");
                            Ok(
                                GurtResponse::internal_server_error().with_string_body(
                                    "Invalid response from API"
                                )
                            )
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON response: {}", e);
                        Ok(
                            GurtResponse::internal_server_error().with_string_body(
                                "Failed to parse API response"
                            )
                        )
                    }
                }
            } else if response.status() == 404 {
                info!("Short URL not found: {}", short_url);
                let not_found_html =
                    format!(r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>URL Not Found</title>
                    </head>
                    <body>
                        <h1>URL Not Found</h1>
                        <p>The short URL <code>{}</code> was not found.</p>
                        <p><a href="gurt://127.0.0.1:4878/">Create a new short URL</a></p>
                    </body>
                    </html>
                "#, short_url);
                Ok(
                    GurtResponse::not_found()
                        .with_header("content-type", "text/html; charset=utf-8")
                        .with_string_body(not_found_html)
                )
            } else {
                error!("API request failed with status: {}", response.status());
                Ok(GurtResponse::internal_server_error().with_string_body("API request failed"))
            }
        }
        Err(e) => {
            error!("Failed to connect to API: {}", e);
            Ok(GurtResponse::internal_server_error().with_string_body("Failed to connect to API"))
        }
    }
}
