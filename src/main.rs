use axum::{
    extract::{Multipart, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{
    net::SocketAddr,
    path::Path,
    sync::{Arc, OnceLock, RwLock},
};
use tokio::fs;
use tokio::sync::RwLock as AsyncRwLock;
use tower_http::services::ServeDir;
use uuid::Uuid;

static DB_POOL: OnceLock<PgPool> = OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct User {
    id: String,
    username: String,
    email: String,
    password_hash: String,
    avatar_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    id: String,
    user_id: String,
    token: String,
    expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Story {
    id: String,
    author: String,
    caption: String,
    media_type: String,
    media_url: Option<String>,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchHistoryEntry {
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
pub struct CallSession {
    id: String,
    room_id: String,
    call_type: String,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    duration_ms: i64,
    participants: Vec<String>,
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DMMessage {
    id: String,
    from: String,
    to: String,
    text: String,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    id: String,
    name: String,
    description: String,
    owner: String,
    members: Vec<String>,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceRoom {
    id: String,
    name: String,
    participants: Vec<String>,
    status: String,
    created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reel {
    id: String,
    title: String,
    author: String,
    url: String,
    duration: String,
    views: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationItem {
    id: String,
    title: String,
    text: String,
    created_at: DateTime<Utc>,
    read: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppData {
    users: Vec<User>,
    sessions: Vec<Session>,
    stories: Vec<Story>,
    search_history: Vec<SearchHistoryEntry>,
    call_sessions: Vec<CallSession>,
    dms: Vec<DMMessage>,
    groups: Vec<Group>,
    voice_rooms: Vec<VoiceRoom>,
    reels: Vec<Reel>,
    notifications: Vec<NotificationItem>,
    active_user: Option<User>,
}

type SharedState = Arc<AsyncRwLock<AppData>>;

#[derive(Clone, Debug, Deserialize)]
struct LoginRequest {
    username: String,
    email: String,
    password: String,
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
struct DMRequest {
    to: String,
    text: String,
}

#[derive(Clone, Debug, Deserialize)]
struct GroupRequest {
    name: String,
    description: String,
    owner: String,
    members: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct VoiceRoomRequest {
    name: String,
    participants: Vec<String>,
    status: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ReelRequest {
    title: String,
    author: String,
    url: String,
    duration: String,
    views: i64,
}

#[derive(Clone, Debug, Deserialize)]
struct NotificationRequest {
    title: String,
    text: String,
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

async fn connect_database() -> Option<PgPool> {
    let db_url = match std::env::var("DATABASE_URL") {
        Ok(value) => value,
        Err(_) => return None,
    };

    match PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await
    {
        Ok(pool) => {
            println!("Connected to PostgreSQL.");
            match sqlx::migrate!("./migrations").run(&pool).await {
                Ok(_) => println!("Database migrations applied."),
                Err(err) => eprintln!("Database migration failed: {err}"),
            }
            Some(pool)
        }
        Err(err) => {
            eprintln!("Database unavailable: {err}");
            None
        }
    }
}

async fn db_status() -> impl IntoResponse {
    let connected = DB_POOL.get().is_some();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "database_connected": connected,
            "mode": if connected { "postgres" } else { "memory" },
            "message": if connected {
                "Connected to PostgreSQL and migrations applied."
            } else {
                "DATABASE_URL not set; running in in-memory demo mode."
            }
        })),
    )
}

fn valid_email(email: &str) -> bool {
    let trimmed = email.trim();
    if trimmed.is_empty() || !trimmed.contains('@') {
        return false;
    }

    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return false;
    }

    true
}

fn hash_password(password: &str) -> String {
    format!("sha256:{}", password)
}

fn session_cookie(token: String) -> Cookie<'static> {
    Cookie::build(("rustcord_session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(30))
        .finish()
}

fn clear_cookie() -> Cookie<'static> {
    Cookie::build(("rustcord_session", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(0))
        .finish()
}

fn format_duration_ms(duration_ms: i64) -> String {
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
    let sample_user = User {
        id: "user-osxcel".to_string(),
        username: "OsxCel".to_string(),
        email: "osxcel@example.com".to_string(),
        password_hash: hash_password("demo123"),
        avatar_url: None,
    };

    AppData {
        users: vec![sample_user.clone()],
        sessions: vec![],
        stories: vec![
            Story {
                id: Uuid::new_v4().to_string(),
                author: "Aylin".to_string(),
                caption: "Yeni haftanın enerjisi!".to_string(),
                media_type: "image".to_string(),
                media_url: Some("https://images.unsplash.com/photo-1493246507139-91e8fad9978e?auto=format&fit=crop&w=800&q=80".to_string()),
                created_at: now - Duration::minutes(18),
                expires_at: now + Duration::hours(23),
            },
            Story {
                id: Uuid::new_v4().to_string(),
                author: "Burak".to_string(),
                caption: "Rust ile üretim zamanı.".to_string(),
                media_type: "text".to_string(),
                media_url: None,
                created_at: now - Duration::minutes(47),
                expires_at: now + Duration::hours(22),
            },
        ],
        search_history: vec![
            SearchHistoryEntry {
                id: Uuid::new_v4().to_string(),
                scope: "dm".to_string(),
                entity_id: "user-aylin".to_string(),
                entity_label: "Aylin".to_string(),
                query: "gece planı".to_string(),
                started_at: now - Duration::minutes(15),
                ended_at: now - Duration::minutes(14),
                duration_ms: 30_000,
                result_count: 3,
            },
            SearchHistoryEntry {
                id: Uuid::new_v4().to_string(),
                scope: "group".to_string(),
                entity_id: "group-rust".to_string(),
                entity_label: "Rust Development".to_string(),
                query: "voice chat".to_string(),
                started_at: now - Duration::hours(2),
                ended_at: now - Duration::hours(1),
                duration_ms: 3_600_000,
                result_count: 12,
            },
        ],
        call_sessions: vec![CallSession {
            id: Uuid::new_v4().to_string(),
            room_id: "room-dev-01".to_string(),
            call_type: "voice".to_string(),
            started_at: now - Duration::minutes(22),
            ended_at: Some(now),
            duration_ms: 22 * 60 * 1000,
            participants: vec!["Aylin".to_string(), "Burak".to_string(), "Ceren".to_string()],
            status: "completed".to_string(),
        }],
        dms: vec![
            DMMessage {
                id: Uuid::new_v4().to_string(),
                from: "Aylin".to_string(),
                to: "OsxCel".to_string(),
                text: "Bugün 19.00 da grup odasında buluşalım mı?".to_string(),
                created_at: now - Duration::minutes(6),
            },
            DMMessage {
                id: Uuid::new_v4().to_string(),
                from: "OsxCel".to_string(),
                to: "Aylin".to_string(),
                text: "Olur, hazırım.".to_string(),
                created_at: now - Duration::minutes(5),
            },
        ],
        groups: vec![
            Group {
                id: "group-rust".to_string(),
                name: "Rust Development".to_string(),
                description: "Rust, axum, postgres, realtime, architecture".to_string(),
                owner: "OsxCel".to_string(),
                members: vec!["OsxCel".to_string(), "Aylin".to_string(), "Burak".to_string()],
                created_at: now - Duration::days(12),
            },
            Group {
                id: "group-design".to_string(),
                name: "Design Crew".to_string(),
                description: "Arayüz, UX, motion ve prototyping".to_string(),
                owner: "Aylin".to_string(),
                members: vec!["Aylin".to_string(), "OsxCel".to_string(), "Selin".to_string()],
                created_at: now - Duration::days(9),
            },
        ],
        voice_rooms: vec![
            VoiceRoom {
                id: Uuid::new_v4().to_string(),
                name: "Rust Labs".to_string(),
                participants: vec!["OsxCel".to_string(), "Burak".to_string()],
                status: "live".to_string(),
                created_at: now - Duration::minutes(10),
            },
            VoiceRoom {
                id: Uuid::new_v4().to_string(),
                name: "Design Review".to_string(),
                participants: vec!["Aylin".to_string(), "Selin".to_string()],
                status: "idle".to_string(),
                created_at: now - Duration::hours(1),
            },
        ],
        reels: vec![
            Reel {
                id: Uuid::new_v4().to_string(),
                title: "Rust + Axum kısa demo".to_string(),
                author: "OsxCel".to_string(),
                url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
                duration: "00:35".to_string(),
                views: 25900,
            },
            Reel {
                id: Uuid::new_v4().to_string(),
                title: "Product prototype preview".to_string(),
                author: "Aylin".to_string(),
                url: "https://www.youtube.com/watch?v=ysz5S6PUM-U".to_string(),
                duration: "00:52".to_string(),
                views: 19800,
            },
        ],
        notifications: vec![
            NotificationItem {
                id: Uuid::new_v4().to_string(),
                title: "Yeni DM".to_string(),
                text: "Aylin senden yeni mesaj bıraktı.".to_string(),
                created_at: now - Duration::minutes(2),
                read: false,
            },
            NotificationItem {
                id: Uuid::new_v4().to_string(),
                title: "Sesli odada katılım".to_string(),
                text: "Burak Rust Labs odasına katıldı.".to_string(),
                created_at: now - Duration::minutes(8),
                read: true,
            },
        ],
        active_user: Some(sample_user),
    }
}

async fn current_user_from_cookie(state: &SharedState, cookie: Option<&str>) -> Option<User> {
    let token = cookie?;
    let data = state.read().await;
    let session = data.sessions.iter().find(|s| s.token == token)?;
    if session.expires_at < Utc::now() {
        return None;
    }
    data.users.iter().find(|u| u.id == session.user_id).cloned()
}

async fn healthcheck() -> &'static str {
    "ok"
}

async fn login_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Response {
    if req.username.trim().is_empty() || req.email.trim().is_empty() || req.password.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": "Tüm alanlar zorunlu." })),
        )
            .into_response();
    }

    if !valid_email(&req.email) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": "Geçersiz e-posta adresi." })),
        )
            .into_response();
    }

    let mut data = state.write().await;
    let mut user = data
        .users
        .iter()
        .find(|u| u.username == req.username.trim() || u.email == req.email.trim())
        .cloned();

    if user.is_none() {
        user = Some(User {
            id: Uuid::new_v4().to_string(),
            username: req.username.trim().to_string(),
            email: req.email.trim().to_string(),
            password_hash: hash_password(&req.password),
            avatar_url: None,
        });
        if let Some(new_user) = user.clone() {
            data.users.push(new_user);
        }
    }

    let user_obj = user.clone().unwrap();
    let token = Uuid::new_v4().to_string();
    let session = Session {
        id: Uuid::new_v4().to_string(),
        user_id: user_obj.id.clone(),
        token: token.clone(),
        expires_at: Utc::now() + Duration::days(30),
    };

    data.sessions.retain(|s| s.user_id != user_obj.id);
    data.sessions.push(session);
    data.active_user = Some(user_obj.clone());

    let cookie = session_cookie(token);
    let mut response = Json(serde_json::json!({
        "success": true,
        "message": "Oturum açıldı.",
        "user": user_obj
    }))
    .into_response();

    response
        .headers_mut()
        .append("Set-Cookie", HeaderValue::from_str(&cookie.encoded().to_string()).unwrap());

    response
}

async fn me_handler(State(state): State<SharedState>, jar: CookieJar) -> Response {
    let cookie = jar.get("rustcord_session").map(|c| c.value().to_string());
    match current_user_from_cookie(&state, cookie.as_deref()).await {
        Some(user) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "user": user
            })),
        )
            .into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "message": "Unauthorized"
            })),
        )
            .into_response(),
    }
}

async fn logout_handler(State(state): State<SharedState>, jar: CookieJar) -> Response {
    let cookie_value = jar.get("rustcord_session").map(|c| c.value().to_string());
    if let Some(token) = cookie_value {
        let mut data = state.write().await;
        data.sessions.retain(|s| s.token != token);
        data.active_user = None;
    }

    let cookie = clear_cookie();
    let mut response = Json(serde_json::json!({
        "success": true,
        "message": "Oturum kapatıldı."
    }))
    .into_response();

    response
        .headers_mut()
        .append("Set-Cookie", HeaderValue::from_str(&cookie.encoded().to_string()).unwrap());

    response
}

async fn state_handler(State(state): State<SharedState>, jar: CookieJar) -> Response {
    let cookie = jar.get("rustcord_session").map(|c| c.value().to_string());
    match current_user_from_cookie(&state, cookie.as_deref()).await {
        Some(user) => {
            let data = state.read().await;
            let mut payload = serde_json::to_value(&*data).unwrap();
            if let Some(obj) = payload.get_mut("active_user") {
                *obj = serde_json::to_value(&user).unwrap();
            }
            Json(payload).into_response()
        }
        None => (
            StatusCode::OK,
            Json(serde_json::json!({
                "stories": [],
                "search_history": [],
                "call_sessions": [],
                "dms": [],
                "groups": [],
                "voice_rooms": [],
                "reels": [],
                "notifications": [],
                "active_user": null,
                "users": []
            })),
        )
            .into_response(),
    }
}

async fn add_search_handler(
    State(state): State<SharedState>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let started = chrono::DateTime::parse_from_rfc3339(&req.started_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let ended = chrono::DateTime::parse_from_rfc3339(&req.ended_at)
        .map(|dt| dt.with_timezone(&Utc))
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
            "search_history": data.search_history
        })),
    )
}

async fn add_story_handler(
    State(state): State<SharedState>,
    Json(req): Json<StoryRequest>,
) -> impl IntoResponse {
    if req.caption.trim().is_empty() && req.media_url.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "success": false,
                "message": "Hikaye içeriği boş olamaz."
            })),
        );
    }

    let mut data = state.write().await;
    let now = Utc::now();
    let author = data.active_user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "Ben".to_string());

    data.stories.insert(
        0,
        Story {
            id: Uuid::new_v4().to_string(),
            author,
            caption: req.caption.trim().to_string(),
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

async fn add_dm_handler(
    State(state): State<SharedState>,
    Json(req): Json<DMRequest>,
) -> impl IntoResponse {
    let mut data = state.write().await;
    let sender = data.active_user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "OsxCel".to_string());

    data.dms.push(DMMessage {
        id: Uuid::new_v4().to_string(),
        from: sender.clone(),
        to: req.to.clone(),
        text: req.text,
        created_at: Utc::now(),
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "DM gönderildi.",
            "messages": data.dms
        })),
    )
}

async fn add_group_handler(
    State(state): State<SharedState>,
    Json(req): Json<GroupRequest>,
) -> impl IntoResponse {
    let mut data = state.write().await;
    let members = if req.members.is_empty() {
        vec![req.owner.clone()]
    } else {
        req.members.clone()
    };

    let group = Group {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        description: req.description,
        owner: req.owner,
        members,
        created_at: Utc::now(),
    };

    data.groups.insert(0, group.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Grup oluşturuldu.",
            "groups": data.groups
        })),
    )
}

async fn add_voice_room_handler(
    State(state): State<SharedState>,
    Json(req): Json<VoiceRoomRequest>,
) -> impl IntoResponse {
    let mut data = state.write().await;
    let room = VoiceRoom {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        participants: req.participants,
        status: req.status,
        created_at: Utc::now(),
    };

    data.voice_rooms.insert(0, room.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Sesli oda oluşturuldu.",
            "voice_rooms": data.voice_rooms
        })),
    )
}

async fn add_reel_handler(
    State(state): State<SharedState>,
    Json(req): Json<ReelRequest>,
) -> impl IntoResponse {
    let mut data = state.write().await;
    let reel = Reel {
        id: Uuid::new_v4().to_string(),
        title: req.title,
        author: req.author,
        url: req.url,
        duration: req.duration,
        views: req.views,
    };

    data.reels.insert(0, reel.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Reel eklendi.",
            "reels": data.reels
        })),
    )
}

async fn add_notification_handler(
    State(state): State<SharedState>,
    Json(req): Json<NotificationRequest>,
) -> impl IntoResponse {
    let mut data = state.write().await;
    let item = NotificationItem {
        id: Uuid::new_v4().to_string(),
        title: req.title,
        text: req.text,
        created_at: Utc::now(),
        read: false,
    };

    data.notifications.insert(0, item.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Bildirim oluşturuldu.",
            "notifications": data.notifications
        })),
    )
}

async fn add_call_session_handler(
    State(state): State<SharedState>,
    Json(req): Json<CallSessionRequest>,
) -> impl IntoResponse {
    let started = chrono::DateTime::parse_from_rfc3339(&req.started_at)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    let ended = req
        .ended_at
        .as_ref()
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let duration_ms = match ended {
        Some(end_dt) => end_dt.timestamp_millis() - started.timestamp_millis(),
        None => 0,
    };

    let call = CallSession {
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
    data.call_sessions.insert(0, call.clone());

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Sesli çağrı kaydı eklendi.",
            "call_sessions": data.call_sessions,
            "duration_human": format_duration_ms(call.duration_ms)
        })),
    )
}

async fn upload_avatar_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> Response {
    let cookie = jar.get("rustcord_session").map(|c| c.value().to_string());
    let Some(user) = current_user_from_cookie(&state, cookie.as_deref()).await else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "message": "Unauthorized"
            })),
        )
            .into_response();
    };

    let mut avatar_url: Option<String> = None;
    let mut found_file = false;
    while let Some(field) = multipart.next_field().await.transpose().ok().flatten() {
        let name = field.name().unwrap_or("avatar").to_string();
        if name == "avatar" {
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            let upload_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("uploads").join("avatars");
            if !upload_root.exists() {
                let _ = fs::create_dir_all(&upload_root).await;
            }
            let file_name = format!("{}.png", user.id);
            let file_path = upload_root.join(file_name);
            let _ = fs::write(file_path, &bytes).await;
            avatar_url = Some(format!("/uploads/avatars/{}.png", user.id));
            found_file = true;
            break;
        }
    }

    if !found_file || avatar_url.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "message": "Avatar dosyası bulunamadı." })),
        )
            .into_response();
    }

    let mut data = state.write().await;
    if let Some(user_ref) = data.users.iter_mut().find(|u| u.id == user.id) {
        user_ref.avatar_url = avatar_url.clone();
    }
    if let Some(active) = data.active_user.as_mut() {
        if active.id == user.id {
            active.avatar_url = avatar_url.clone();
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Avatar güncellendi.",
            "avatar_url": avatar_url
        })),
    )
        .into_response()
}

#[tokio::main]
async fn main() {
    if let Some(pool) = connect_database().await {
        let _ = DB_POOL.set(pool);
    }

    let app_state: SharedState = Arc::new(AsyncRwLock::new(seed_data()));

    let static_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("static")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static"));
    let upload_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("uploads");

    let static_service = ServeDir::new(static_dir).append_index_html_on_directories(true);
    let upload_service = ServeDir::new(upload_dir);

    let app = Router::new()
        .route("/health", get(healthcheck))
        .route("/api/db-status", get(db_status))
        .route("/api/login", post(login_handler))
        .route("/api/logout", post(logout_handler))
        .route("/api/me", get(me_handler))
        .route("/api/state", get(state_handler))
        .route("/api/search-history", post(add_search_handler))
        .route("/api/stories", post(add_story_handler))
        .route("/api/dm", post(add_dm_handler))
        .route("/api/groups", post(add_group_handler))
        .route("/api/voice-rooms", post(add_voice_room_handler))
        .route("/api/reels", post(add_reel_handler))
        .route("/api/notifications", post(add_notification_handler))
        .route("/api/call-sessions", post(add_call_session_handler))
        .route("/api/profile/avatar", post(upload_avatar_handler))
        .nest_service("/uploads", upload_service)
        .fallback_service(static_service)
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("RustCord Red listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
