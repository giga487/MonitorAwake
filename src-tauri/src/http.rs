use crate::{core::Action, service::SharedService};
use axum::{
    Json, Router,
    body::Body,
    extract::{DefaultBodyLimit, Request, State},
    http::{StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use rust_embed::RustEmbed;
use std::sync::Arc;

#[derive(RustEmbed)]
#[folder = "../dist/"]
struct Assets;

#[derive(Clone)]
pub struct HttpState {
    pub service: SharedService,
    pub token: Arc<str>,
    pub authority: Arc<str>,
}

pub fn router(state: HttpState) -> Router {
    Router::new()
        .route("/api/state", get(snapshot))
        .route("/api/action", post(action))
        .fallback(asset)
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), guard))
        .with_state(state)
}

async fn guard(State(state): State<HttpState>, request: Request, next: Next) -> Response {
    let headers = request.headers();
    if headers.get(header::HOST).and_then(|v| v.to_str().ok()) != Some(state.authority.as_ref()) {
        return error(StatusCode::FORBIDDEN, "Host non consentito.");
    }
    let origin = format!("http://{}", state.authority);
    if let Some(value) = headers.get(header::ORIGIN)
        && value.to_str().ok() != Some(origin.as_str())
    {
        return error(StatusCode::FORBIDDEN, "Origine non consentita.");
    }
    if request.uri().path().starts_with("/api/") {
        let expected = format!("Bearer {}", state.token);
        if headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            != Some(expected.as_str())
        {
            return error(
                StatusCode::UNAUTHORIZED,
                "Apri la configurazione dalla mascotte per accedere.",
            );
        }
    }
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(header::REFERRER_POLICY, "no-referrer".parse().unwrap());
    headers.insert(header::CONTENT_SECURITY_POLICY,
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'".parse().unwrap());
    response
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}

async fn snapshot(State(state): State<HttpState>) -> Response {
    match state.service.lock() {
        Ok(service) => Json(service.snapshot()).into_response(),
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "Motore non disponibile."),
    }
}

async fn action(State(state): State<HttpState>, Json(action): Json<Action>) -> Response {
    match state.service.lock() {
        Ok(mut service) => match service.dispatch(action) {
            Ok(result) => Json(result).into_response(),
            Err(message) => error(StatusCode::BAD_REQUEST, &message),
        },
        Err(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "Motore non disponibile."),
    }
}

async fn asset(request: Request) -> Response {
    if request.method() != axum::http::Method::GET && request.method() != axum::http::Method::HEAD {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let path = request.uri().path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    let Some(file) = Assets::get(path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();
    let body = if request.method() == axum::http::Method::HEAD {
        Body::empty()
    } else {
        Body::from(file.data.into_owned())
    };
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .body(body)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::Service;
    use std::sync::Mutex;
    use tower::ServiceExt;

    #[tokio::test]
    async fn protects_state_and_mutations_and_serves_embedded_page() {
        let dir = tempfile::tempdir().unwrap();
        let service = Arc::new(Mutex::new(
            Service::load(dir.path().join("state.json")).unwrap(),
        ));
        let app = router(HttpState {
            service,
            token: Arc::from("test-secret"),
            authority: Arc::from("127.0.0.1:1234"),
        });
        for (host, token, origin, expected) in [
            (
                "evil.test",
                "Bearer test-secret",
                None,
                StatusCode::FORBIDDEN,
            ),
            ("127.0.0.1:1234", "", None, StatusCode::UNAUTHORIZED),
            (
                "127.0.0.1:1234",
                "Bearer wrong",
                None,
                StatusCode::UNAUTHORIZED,
            ),
            (
                "127.0.0.1:1234",
                "Bearer test-secret",
                Some("https://evil.test"),
                StatusCode::FORBIDDEN,
            ),
            (
                "127.0.0.1:1234",
                "Bearer test-secret",
                Some("null"),
                StatusCode::FORBIDDEN,
            ),
            (
                "127.0.0.1:1234",
                "Bearer test-secret",
                Some("http://127.0.0.1:1234"),
                StatusCode::OK,
            ),
        ] {
            let mut request = Request::builder()
                .uri("/api/state")
                .header("Host", host)
                .header("Authorization", token);
            if let Some(origin) = origin {
                request = request.header("Origin", origin);
            }
            let response = app
                .clone()
                .oneshot(request.body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
            assert!(
                !response
                    .headers()
                    .contains_key("access-control-allow-origin")
            );
        }
        let request = Request::builder()
            .uri("/api/action")
            .method("POST")
            .header("Host", "127.0.0.1:1234")
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer test-secret")
            .body(Body::from(
                r#"{"type":"add_timer","name":"Test","seconds":30,"keep_awake":false}"#,
            ))
            .unwrap();
        assert_eq!(
            app.clone().oneshot(request).await.unwrap().status(),
            StatusCode::OK
        );
        let request = Request::builder()
            .uri("/")
            .header("Host", "127.0.0.1:1234")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    }
}
