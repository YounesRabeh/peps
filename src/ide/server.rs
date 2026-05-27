//! Local HTTP server for the Peps browser IDE.

use std::{net::SocketAddr, path::PathBuf};

use axum::{
    extract::Json,
    response::{Html, IntoResponse},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{diagnostic::Diagnostic, run_source};

const DEFAULT_ADDR: &str = "127.0.0.1:5179";

#[derive(Debug, Deserialize)]
pub struct RunRequest {
    pub source: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct RunResponse {
    pub ok: bool,
    pub output: Vec<String>,
    pub diagnostics: Vec<IdeDiagnostic>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct IdeDiagnostic {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

/// Start the local IDE server and serve the built frontend from `ide/dist`.
pub async fn run() -> anyhow::Result<()> {
    let addr: SocketAddr = DEFAULT_ADDR.parse()?;
    let dist_dir = PathBuf::from("ide/dist");

    if !dist_dir.exists() {
        eprintln!(
            "warning: ide/dist was not found. Build the frontend first:\n  cd ide\n  npm install\n  npm run build"
        );
    }

    let app = router(dist_dir);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Peps IDE running at http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

pub fn router(dist_dir: PathBuf) -> Router {
    let router = Router::new()
        .route("/api/run", post(run_handler))
        .layer(CorsLayer::permissive());

    if dist_dir.exists() {
        router.fallback_service(ServeDir::new(dist_dir).append_index_html_on_directories(true))
    } else {
        router.fallback(missing_frontend_handler)
    }
}

pub async fn run_handler(Json(request): Json<RunRequest>) -> Json<RunResponse> {
    // Keep the IDE thin: compiler and runtime behavior lives behind run_source.
    match run_source(&request.source) {
        Ok(output) => Json(RunResponse {
            ok: true,
            output,
            diagnostics: Vec::new(),
        }),
        Err(error) => Json(RunResponse {
            ok: false,
            output: error.output,
            diagnostics: error
                .diagnostics
                .iter()
                .map(IdeDiagnostic::from)
                .collect(),
        }),
    }
}

async fn missing_frontend_handler() -> impl IntoResponse {
    Html(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Peps IDE</title>
    <style>
      body {
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
        background: #101318;
        color: #f3f7ff;
        font-family: system-ui, sans-serif;
      }
      main {
        max-width: 720px;
        padding: 32px;
      }
      code, pre {
        font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      }
      pre {
        background: #181d25;
        border: 1px solid #2b3443;
        border-radius: 8px;
        padding: 16px;
      }
    </style>
  </head>
  <body>
    <main>
      <h1>Peps IDE frontend is not built yet</h1>
      <p>Build the browser app first, then restart the IDE server:</p>
      <pre>cd ide
npm install
npm run build
cd ..
cargo run --bin peps-ide</pre>
    </main>
  </body>
</html>"#,
    )
}

impl From<&Diagnostic> for IdeDiagnostic {
    fn from(diagnostic: &Diagnostic) -> Self {
        Self {
            message: diagnostic.message.clone(),
            line: diagnostic.span.map(|span| span.line),
            column: diagnostic.span.map(|span| span.column),
            start: diagnostic.span.map(|span| span.start),
            end: diagnostic.span.map(|span| span.end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{run_handler, RunRequest};
    use axum::{extract::Json, response::IntoResponse};

    #[tokio::test]
    async fn api_run_success() {
        let response = run_handler(Json(RunRequest {
            source: "🐶 🟰 5️⃣ 🔚\n📢 🐶 🔚".to_string(),
        }))
        .await
        .0;

        assert!(response.ok);
        assert_eq!(response.output, vec!["5".to_string()]);
        assert!(response.diagnostics.is_empty());
    }

    #[tokio::test]
    async fn api_run_diagnostics() {
        let response = run_handler(Json(RunRequest {
            source: "📢 🐶 🔚".to_string(),
        }))
        .await
        .0;

        assert!(!response.ok);
        assert!(!response.diagnostics.is_empty());
    }

    #[tokio::test]
    async fn missing_frontend_page_is_helpful() {
        let response = super::missing_frontend_handler().await.into_response();
        assert!(response.status().is_success());
    }
}
