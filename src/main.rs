use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: String,
    username: String,
    email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Story {
    id: String,
    author: String,
    caption: String,
    media_type: String,
    media_url: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SearchHistoryEntry {
    id: String,
    scope: String,
    entity_id: String,
    entity_label: String,
    query: String,
    started_at: DateTime<Utc>,
    ended_at: DateTime<Utc>,
    duration_ms: i64,
    result_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CallSession {
    id: String,
    room_id: String,
    call_type: String,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    duration_ms: i64,
    participants: Vec<String>,
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct AppData {
    users: Vec<User>,
    stories: Vec<Story>,
    search_history: Vec<SearchHistoryEntry>,
    call_sessions: Vec<CallSession>,
    active_user: Option<User>,
}

type SharedState = Arc<RwLock<AppData>>;

#[derive(Clone, Debug, Deserialize)]
struct LoginRequest {
    username: String,
    email: String,
}

#[derive(Clone, Debug, Deserialize)]
struct SearchRequest {
    scope: String,
    entity_id: String,
    entity_label: String,
    query: String,
    started_at: String,
    ended_at: String,
    result_count: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct StoryRequest {
    caption: String,
    media_url: Option<String>,
    media_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct CallSessionRequest {
    room_id: String,
    call_type: String,
    started_at: String,
    ended_at: Option<String>,
    participants: Vec<String>,
    status: String,
}

fn fmt_duration(duration_ms: i64) -> String {
    let total_seconds = (duration_ms / 1000) as i64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{} saat {} dk {} sn", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{} dk {} sn", minutes, seconds)
    } else {
        format!("{} sn", seconds)
    }
}

fn seed_data() -> AppData {
    let now = Utc::now();
    let sample_search_1 = SearchHistoryEntry {
        id: Uuid::new_v4().to_string(),
        scope: "dm".to_string(),
        entity_id: "user-1".to_string(),
        entity_label: "Aylin".to_string(),
        query: "gece planı".to_string(),
        started_at: now - Duration::minutes(15),
        ended_at: now - Duration::minutes(14),
        duration_ms: 30_000,
        result_count: 3,
    };

    let sample_search_2 = SearchHistoryEntry {
        id: Uuid::new_v4().to_string(),
        scope: "group".to_string(),
        entity_id: "group-9".to_string(),
        entity_label: "Rust Development".to_string(),
        query: "voice chat".to_string(),
        started_at: now - Duration::hours(2),
        ended_at: now - Duration::hours(1),
        duration_ms: 3_600_000,
        result_count: 12,
    };

    let sample_call = CallSession {
        id: Uuid::new_v4().to_string(),
        room_id: "room-dev-01".to_string(),
        call_type: "voice".to_string(),
        started_at: now - Duration::minutes(22),
        ended_at: Some(now),
        duration_ms: 22 * 60 * 1000,
        participants: vec!["Aylin".to_string(), "Burak".to_string(), "Ceren".to_string()],
        status: "completed".to_string(),
    };

    AppData {
        users: vec![User {
            id: "user-me".to_string(),
            username: "OsxCel".to_string(),
            email: "osxcel@example.com".to_string(),
        }],
        stories: vec![
            Story {
                id: Uuid::new_v4().to_string(),
                author: "Aylin".to_string(),
                caption: "Yeni haftanın enerjisi!".to_string(),
                media_type: "image".to_string(),
                media_url: Some("https://images.unsplash.com/photo-1493246507139-91e8fad9978e?auto=format&fit=crop&w=800&q=80".to_string()),
                created_at: now - Duration::minutes(25),
                expires_at: now + Duration::hours(23),
            },
            Story {
                id: Uuid::new_v4().to_string(),
                author: "Burak".to_string(),
                caption: "Yazılımcı sabahı.".to_string(),
                media_type: "text".to_string(),
                media_url: None,
                created_at: now - Duration::minutes(45),
                expires_at: now + Duration::hours(22),
            },
        ],
        search_history: vec![sample_search_1, sample_search_2],
        call_sessions: vec![sample_call],
        active_user: None,
    }
}

async fn healthcheck() -> &'static str {
    "ok"
}

async fn login_handler(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = User {
        id: Uuid::new_v4().to_string(),
        username: req.username.trim().to_string(),
        email: req.email.trim().to_string(),
    };

    let mut data = state.write().await;
    data.active_user = Some(user.clone());
    data.users.push(user);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Oturum açıldı.",
            "user": data.active_user
        })),
    )
}

async fn state_handler(State(state): State<SharedState>) -> impl IntoResponse {
    let data = state.read().await;
    Json(serde_json::to_value(&*data).unwrap())
}

async fn add_search_handler(
    State(state): State<SharedState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let started = chrono::DateTime::parse_from_rfc3339(&req.started_at)
        .map(|v| v.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let ended = chrono::DateTime::parse_from_rfc3339(&req.ended_at)
        .map(|v| v.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let duration_ms = ended.timestamp_millis() - started.timestamp_millis();

    let entry = SearchHistoryEntry {
        id: Uuid::new_v4().to_string(),
        scope: req.scope,
        entity_id: req.entity_id,
        entity_label: req.entity_label,
        query: req.query,
        started_at: started,
        ended_at: ended,
        duration_ms,
        result_count: req.result_count,
    };

    let mut data = state.write().await;
    data.search_history.insert(0, entry);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Arama kaydı eklendi.",
            "history": data.search_history
        })),
    )
}

async fn add_story_handler(
    State(state): State<SharedState>,
    Json(req): Json<StoryRequest>,
) -> impl IntoResponse {
    let caption = req.caption.trim();
    if caption.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "message": "Hikaye metni boş olamaz."
            })),
        );
    }

    let mut data = state.write().await;
    let author = data.active_user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "Ben".to_string());
    let now = Utc::now();

    data.stories.insert(
        0,
        Story {
            id: Uuid::new_v4().to_string(),
            author,
            caption: caption.to_string(),
            media_type: req.media_type.unwrap_or_else(|| "text".to_string()),
            media_url: req.media_url,
            created_at: now,
            expires_at: now + Duration::hours(24),
        },
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Hikaye yayınlandı.",
            "stories": data.stories
        })),
    )
}

async fn add_call_session_handler(
    State(state): State<SharedState>,
    Json(req): Json<CallSessionRequest>,
) -> impl IntoResponse {
    let started = chrono::DateTime::parse_from_rfc3339(&req.started_at)
        .map(|v| v.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let ended = req
        .ended_at
        .as_ref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|v| v.with_timezone(&Utc));

    let duration_ms = match ended {
        Some(end_dt) => end_dt.timestamp_millis() - started.timestamp_millis(),
        None => 0,
    };

    let session = CallSession {
        id: Uuid::new_v4().to_string(),
        room_id: req.room_id,
        call_type: req.call_type,
        started_at: started,
        ended_at: ended,
        duration_ms,
        participants: req.participants,
        status: req.status,
    };

    let mut data = state.write().await;
    data.call_sessions.insert(0, session);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Sesli arama kaydı tutuldu.",
            "calls": data.call_sessions
        })),
    )
}

#[tokio::main]
async fn main() {
    let app_state: SharedState = Arc::new(RwLock::new(seed_data()));

    let static_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("static")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"));

    let static_service = ServeDir::new(static_dir).append_index_html_on_directories(true);

    let app = Router::new()
        .route("/health", get(healthcheck))
        .route("/api/login", post(login_handler))
        .route("/api/state", get(state_handler))
        .route("/api/search-history", post(add_search_handler))
        .route("/api/stories", post(add_story_handler))
        .route("/api/call-sessions", post(add_call_session_handler))
        .fallback_service(static_service)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("RustCord Red listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
