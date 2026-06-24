use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, put};
use axum::{Json, Router};

use serde::Deserialize;

use crate::config::{read_ui, update_theme};
use crate::store::NotesStore;

pub struct AppState {
    pub store: NotesStore,
    pub config_path: PathBuf,
}

pub type SharedState = Arc<AppState>;

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/notes", get(list_notes))
        .route("/api/notes/{date}", get(get_note).put(put_note))
        .route("/api/config", get(get_config))
        .route("/api/config/theme", put(put_config_theme))
        .fallback(crate::assets::static_handler)
        .with_state(state)
}

async fn list_notes(State(state): State<SharedState>) -> impl IntoResponse {
    match state.store.list_dates() {
        Ok(dates) => Json(dates).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_note(State(state): State<SharedState>, Path(date): Path<String>) -> impl IntoResponse {
    if state.store.path_for(&date).is_none() {
        return (StatusCode::BAD_REQUEST, "invalid date").into_response();
    }
    match state.store.read_or_create(&date) {
        Ok(content) => (
            [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
            content,
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn put_note(
    State(state): State<SharedState>,
    Path(date): Path<String>,
    body: String,
) -> impl IntoResponse {
    if state.store.path_for(&date).is_none() {
        return (StatusCode::BAD_REQUEST, "invalid date").into_response();
    }
    match state.store.write(&date, &body) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn get_config(State(state): State<SharedState>) -> impl IntoResponse {
    Json(read_ui(&state.config_path))
}

#[derive(Deserialize)]
struct ThemeRequest {
    theme: String,
}

async fn put_config_theme(
    State(state): State<SharedState>,
    Json(req): Json<ThemeRequest>,
) -> impl IntoResponse {
    if req.theme != "light" && req.theme != "dark" {
        return (StatusCode::BAD_REQUEST, "unknown theme").into_response();
    }
    match update_theme(&state.config_path, &req.theme) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tempfile::tempdir;
    use tower::ServiceExt;

    use crate::store::NotesStore;

    fn test_state(tmp: &tempfile::TempDir) -> SharedState {
        let config_path = tmp.path().join("config.toml");
        std::fs::write(&config_path, "[ui]\ntheme = \"light\"\nfont = \"Roboto\"\n").unwrap();
        Arc::new(AppState {
            store: NotesStore::new(tmp.path().join("notes")),
            config_path,
        })
    }

    async fn body_string(resp: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn get_note_materializes_and_returns_markdown() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(&dir));
        let resp = app
            .oneshot(Request::get("/api/notes/2026-06-23").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp).await;
        assert!(body.starts_with("# 2026-06-23-TUE"));
        assert!(dir.path().join("notes").join("2026-06-23.md").exists());
    }

    #[tokio::test]
    async fn get_note_rejects_bad_date() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(&dir));
        let resp = app
            .oneshot(Request::get("/api/notes/not-a-date").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn put_then_list_round_trips() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(&dir));

        let put = app
            .clone()
            .oneshot(
                Request::put("/api/notes/2026-06-23")
                    .body(Body::from("# hi\n"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::NO_CONTENT);

        let list = app
            .oneshot(Request::get("/api/notes").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(body_string(list).await, "[\"2026-06-23\"]");
    }

    #[tokio::test]
    async fn config_endpoint_returns_ui_json() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(&dir));
        let resp = app
            .oneshot(Request::get("/api/config").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = body_string(resp).await;
        assert!(body.contains("\"theme\":\"light\""));
        assert!(body.contains("\"font\":\"Roboto\""));
    }

    #[tokio::test]
    async fn put_theme_persists_and_get_reflects_it() {
        let dir = tempdir().unwrap();
        let state = test_state(&dir);
        let path = state.config_path.clone();
        let app = build_router(state);

        let put = app
            .clone()
            .oneshot(
                Request::put("/api/config/theme")
                    .header("content-type", "application/json")
                    .body(Body::from("{\"theme\":\"dark\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::NO_CONTENT);

        assert!(std::fs::read_to_string(&path).unwrap().contains("theme = \"dark\""));

        let get = app
            .oneshot(Request::get("/api/config").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(body_string(get).await.contains("\"theme\":\"dark\""));
    }

    #[tokio::test]
    async fn put_theme_rejects_unknown_value() {
        let dir = tempdir().unwrap();
        let state = test_state(&dir);
        let path = state.config_path.clone();
        let app = build_router(state);

        let put = app
            .oneshot(
                Request::put("/api/config/theme")
                    .header("content-type", "application/json")
                    .body(Body::from("{\"theme\":\"neon\"}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(put.status(), StatusCode::BAD_REQUEST);
        // File untouched.
        assert!(std::fs::read_to_string(&path).unwrap().contains("theme = \"light\""));
    }

    #[tokio::test]
    async fn serves_spa_index_at_root() {
        let dir = tempdir().unwrap();
        let app = build_router(test_state(&dir));
        let resp = app
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(body_string(resp).await.contains("Slugline"));
    }
}
