//! Local HTTP server for the Peps browser IDE.

use std::{env, net::SocketAddr, path::PathBuf};

use axum::{
    extract::Json,
    response::{Html, IntoResponse},
    routing::post,
    Router,
};
use serde::Deserialize;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::browser::run_source_for_browser;
pub use crate::browser::{IdeDiagnostic, RunResponse};

/// Address used by the local development IDE server.
const DEFAULT_ADDR: &str = "127.0.0.1:5179";
/// Fallback page shown when the built browser frontend is unavailable.
const MISSING_FRONTEND_HTML: &str = include_str!("missing_frontend.html");

/// JSON payload sent by the browser IDE when running Peps source.
#[derive(Debug, Deserialize)]
pub struct RunRequest {
    /// Source code to compile and execute.
    pub source: String,
}

/// Start the local IDE server and serve the built frontend from `ide/dist`.
pub async fn run() -> anyhow::Result<()> {
    let addr: SocketAddr = DEFAULT_ADDR.parse()?;
    let dist_dir = frontend_dist_dir();

    if !dist_dir.exists() {
        eprintln!(
            "warning: IDE frontend assets were not found. Build the frontend first:\n  cd ide\n  pnpm install --frozen-lockfile\n  pnpm run build"
        );
    }

    let app = router(dist_dir);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    let url = format!("http://{}", addr);
    println!("Peps IDE running at {}", url);

    if env::var_os("PEPS_IDE_NO_BROWSER").is_none() {
        println!("Opening browser...");
        match open::that(&url) {
            Ok(_) => println!("Browser open command sent."),
            Err(error) => {
                eprintln!("Could not open browser automatically: {error}");
                eprintln!("Open it manually at: {url}");
            }
        }
    }

    axum::serve(listener, app).await?;
    Ok(())
}

/// Locate frontend assets relative to the workspace or installed binary.
fn frontend_dist_dir() -> PathBuf {
    let workspace_dist = PathBuf::from("ide/dist");
    if workspace_dist.exists() {
        return workspace_dist;
    }

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let split_bundle_dist = exe_dir.join("frontend").join("dist");
            if split_bundle_dist.exists() {
                return split_bundle_dist;
            }

            let bundled_dist = exe_dir.join("ide").join("dist");
            if bundled_dist.exists() {
                return bundled_dist;
            }
        }
    }

    workspace_dist
}

/// Build the IDE HTTP router for the given frontend asset directory.
///
/// The router always exposes `/api/run`. If `dist_dir` exists, static files are
/// served from it; otherwise every non-API route returns the bundled fallback
/// page that explains how to build the frontend.
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

/// Compile and execute Peps source submitted by the browser IDE.
pub async fn run_handler(Json(request): Json<RunRequest>) -> Json<RunResponse> {
    Json(run_source_for_browser(&request.source))
}

/// Return the fallback HTML shown when frontend assets are missing.
async fn missing_frontend_handler() -> impl IntoResponse {
    Html(MISSING_FRONTEND_HTML)
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
            source: "🐶 🟰 1️⃣ 🔚 🐶 🟰 ✅ 🔚 🐶 🟰 🐶 ➕ 1️⃣ 🔚".to_string(),
        }))
        .await
        .0;

        assert!(!response.ok);
        assert!(!response.diagnostics.is_empty());
    }

    #[tokio::test]
    async fn api_run_enforces_the_ide_step_limit() {
        let response = run_handler(Json(RunRequest {
            source: "🔁 ✅ 🔓 🔒".to_string(),
        }))
        .await
        .0;

        assert!(!response.ok);
        assert!(response.diagnostics[0].message.contains("step limit"));
    }

    #[tokio::test]
    async fn missing_frontend_page_is_helpful() {
        let response = super::missing_frontend_handler().await.into_response();
        assert!(response.status().is_success());
    }
}
