//! Authoritative persisted-before-ACK room service. Tokens never occur in game state or SSE.
use crate::{
    catalog,
    model::{Action, Game, View},
};
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    convert::Infallible,
    path::Path as FilePath,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{watch, Mutex as AsyncMutex};
use tower_http::services::{ServeDir, ServeFile};

#[derive(Clone)]
pub struct Store {
    inner: Arc<Inner>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayAudit {
    pub room_id: String,
    pub version: u64,
    pub journal_entries: u64,
    pub status: String,
    pub matches: bool,
    pub persisted_digest: String,
    pub replayed_digest: String,
    pub versions: crate::model::Versions,
}
struct Inner {
    db: Mutex<Connection>,
    rooms: Mutex<HashMap<String, Arc<Room>>>,
}
struct Room {
    game: AsyncMutex<Game>,
    sender: watch::Sender<Arc<Game>>,
}
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub error: String,
    pub message: String,
    pub view: Option<Box<View>>,
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}
impl std::error::Error for ApiError {}
impl ApiError {
    fn bad(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            error: "invalid_action".into(),
            message: message.into(),
            view: None,
        }
    }
    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: "storage_error".into(),
            message: message.into(),
            view: None,
        }
    }
    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            error: "unauthorized".into(),
            message: "需要此房间的座位令牌".into(),
            view: None,
        }
    }
    fn missing() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            error: "room_not_found".into(),
            message: "房间不存在".into(),
            view: None,
        }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({"error":self.error,"message":self.message,"view":self.view})),
        )
            .into_response()
    }
}
impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self {
        tracing::error!(error=%e,"SQLite operation failed");
        Self::internal("持久化失败，操作未确认")
    }
}
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        tracing::error!(error=%e,"state serialization failed");
        Self::internal("状态读取失败")
    }
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoom {
    pub name: String,
    pub mode: String,
    #[serde(default)]
    pub deck_id: String,
    #[serde(default)]
    pub deck_draft: Option<crate::deck::DeckDraft>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinRoom {
    pub invite_code: String,
    pub name: String,
    #[serde(default)]
    pub deck_id: String,
    #[serde(default)]
    pub deck_draft: Option<crate::deck::DeckDraft>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub room_id: String,
    pub invite_code: String,
    pub token: String,
    pub seat: usize,
    pub view: View,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Command {
    pub command_id: String,
    pub expected_version: u64,
    pub action: Action,
}
#[derive(Serialize, Deserialize)]
enum Journal {
    Join {
        name: String,
        deck_id: String,
    },
    JoinWithDeck {
        name: String,
        deck_draft: crate::deck::DeckDraft,
    },
    Command {
        seat: usize,
        command: Command,
    },
}
fn secure(bytes: usize) -> Result<String, ApiError> {
    let mut buf = vec![0; bytes];
    getrandom::fill(&mut buf).map_err(|_| ApiError::internal("安全随机源不可用"))?;
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}
fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
fn name(input: String) -> Result<String, ApiError> {
    let n = input.trim();
    if n.is_empty() || n.chars().count() > 40 {
        return Err(ApiError::bad("昵称长度须为1至40字"));
    }
    Ok(n.into())
}
fn room(game: Game) -> Arc<Room> {
    let (sender, _) = watch::channel(Arc::new(game.clone()));
    Arc::new(Room {
        game: AsyncMutex::new(game),
        sender,
    })
}
impl Store {
    pub fn open(path: impl AsRef<FilePath>) -> Result<Self, ApiError> {
        let connection = Connection::open(path)?;
        connection.busy_timeout(Duration::from_secs(5))?;
        if connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='rooms'",
            [],
            |r| r.get::<_, usize>(0),
        )? > 0
        {
            let mut statement = connection.prepare("SELECT state FROM rooms")?;
            let states = statement.query_map([], |r| r.get::<_, String>(0))?;
            for state in states {
                Game::from_persisted(&state?).map_err(ApiError::internal)?;
            }
        }
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;
          CREATE TABLE IF NOT EXISTS rooms(id TEXT PRIMARY KEY,invite TEXT UNIQUE NOT NULL,initial_state TEXT NOT NULL,state TEXT NOT NULL);
          CREATE TABLE IF NOT EXISTS seats(room_id TEXT NOT NULL REFERENCES rooms(id),seat INTEGER NOT NULL,token_hash TEXT UNIQUE NOT NULL,PRIMARY KEY(room_id,seat));
          CREATE TABLE IF NOT EXISTS commands(room_id TEXT NOT NULL REFERENCES rooms(id),command_id TEXT NOT NULL,seat INTEGER NOT NULL,expected_version INTEGER NOT NULL,action TEXT NOT NULL,response TEXT NOT NULL,PRIMARY KEY(room_id,command_id));
          CREATE TABLE IF NOT EXISTS journal(room_id TEXT NOT NULL REFERENCES rooms(id),version INTEGER NOT NULL,entry TEXT NOT NULL,PRIMARY KEY(room_id,version));")?;
        let mut rooms = HashMap::new();
        {
            let mut statement = connection.prepare("SELECT id,state FROM rooms")?;
            let mut rows = statement.query([])?;
            while let Some(row) = rows.next()? {
                let id: String = row.get(0)?;
                let state: String = row.get(1)?;
                let game = Game::from_persisted(&state).map_err(ApiError::internal)?;
                if game.versions.rules != catalog::RULES_VERSION
                    || game.versions.card_pool != catalog::POOL_VERSION
                    || game.versions.engine != catalog::ENGINE_VERSION
                {
                    return Err(ApiError::internal(format!(
                        "房间 {id} 的固定版本不兼容；请使用对应引擎启动"
                    )));
                }
                rooms.insert(id, room(game));
            }
        }
        Ok(Self {
            inner: Arc::new(Inner {
                db: Mutex::new(connection),
                rooms: Mutex::new(rooms),
            }),
        })
    }
    fn lookup(&self, id: &str) -> Result<Arc<Room>, ApiError> {
        self.inner
            .rooms
            .lock()
            .unwrap()
            .get(id)
            .cloned()
            .ok_or_else(ApiError::missing)
    }
    pub fn authenticate(&self, id: &str, token: &str) -> Result<usize, ApiError> {
        let db = self.inner.db.lock().unwrap();
        let seat: Option<usize> = db
            .query_row(
                "SELECT seat FROM seats WHERE room_id=?1 AND token_hash=?2",
                params![id, token_hash(token)],
                |r| r.get(0),
            )
            .optional()?;
        seat.ok_or_else(ApiError::unauthorized)
    }
    pub async fn create(&self, request: CreateRoom) -> Result<Session, ApiError> {
        let id = secure(12)?;
        let invite = secure(6)?.to_uppercase();
        let token = secure(32)?;
        let mut seed = [0u8; 8];
        getrandom::fill(&mut seed).map_err(|_| ApiError::internal("安全随机源不可用"))?;
        let n = name(request.name)?;
        let game = if let Some(draft) = request.deck_draft {
            Game::new_with_deck(
                id.clone(),
                invite.clone(),
                request.mode,
                n,
                draft,
                u64::from_le_bytes(seed),
            )
        } else {
            Game::new(
                id.clone(),
                invite.clone(),
                request.mode,
                n,
                request.deck_id,
                u64::from_le_bytes(seed),
            )
        }
        .map_err(ApiError::bad)?;
        let serialized = serde_json::to_string(&game)?;
        let view = game.view(0);
        {
            let mut db = self.inner.db.lock().unwrap();
            let tx = db.transaction()?;
            tx.execute(
                "INSERT INTO rooms(id,invite,initial_state,state) VALUES(?1,?2,?3,?3)",
                params![id, invite, serialized],
            )?;
            tx.execute(
                "INSERT INTO seats(room_id,seat,token_hash) VALUES(?1,0,?2)",
                params![id, token_hash(&token)],
            )?;
            tx.commit()?;
        }
        self.inner
            .rooms
            .lock()
            .unwrap()
            .insert(id.clone(), room(game));
        Ok(Session {
            room_id: id,
            invite_code: invite,
            token,
            seat: 0,
            view,
        })
    }
    pub async fn join(&self, request: JoinRoom) -> Result<Session, ApiError> {
        let invite = request.invite_code.trim().to_uppercase();
        let id: Option<String> = {
            let db = self.inner.db.lock().unwrap();
            db.query_row("SELECT id FROM rooms WHERE invite=?1", [&invite], |r| {
                r.get(0)
            })
            .optional()?
        };
        let id = id.ok_or_else(ApiError::missing)?;
        let target = self.lookup(&id)?;
        let mut current = target.game.lock().await;
        let mut next = current.clone();
        let n = name(request.name)?;
        let (seat, journal) = if let Some(draft) = request.deck_draft {
            let seat = next
                .join_with_deck(n.clone(), draft.clone())
                .map_err(ApiError::bad)?;
            (
                seat,
                Journal::JoinWithDeck {
                    name: n,
                    deck_draft: draft,
                },
            )
        } else {
            let seat = next
                .join(n.clone(), request.deck_id.clone())
                .map_err(ApiError::bad)?;
            (
                seat,
                Journal::Join {
                    name: n,
                    deck_id: request.deck_id,
                },
            )
        };
        let token = secure(32)?;
        let serialized = serde_json::to_string(&next)?;
        let entry = serde_json::to_string(&journal)?;
        {
            let mut db = self.inner.db.lock().unwrap();
            let tx = db.transaction()?;
            tx.execute(
                "UPDATE rooms SET state=?2 WHERE id=?1",
                params![id, serialized],
            )?;
            tx.execute(
                "INSERT INTO seats(room_id,seat,token_hash) VALUES(?1,?2,?3)",
                params![id, seat, token_hash(&token)],
            )?;
            tx.execute(
                "INSERT INTO journal(room_id,version,entry) VALUES(?1,?2,?3)",
                params![id, next.version, entry],
            )?;
            tx.commit()?;
        }
        *current = next;
        let view = current.view(seat);
        target.sender.send_replace(Arc::new(current.clone()));
        Ok(Session {
            room_id: id,
            invite_code: invite,
            token,
            seat,
            view,
        })
    }
    pub async fn state(&self, id: &str, token: &str) -> Result<View, ApiError> {
        let seat = self.authenticate(id, token)?;
        let target = self.lookup(id)?;
        let current = target.game.lock().await;
        Ok(current.view(seat))
    }
    pub async fn command(&self, id: &str, token: &str, command: Command) -> Result<View, ApiError> {
        if command.command_id.is_empty() || command.command_id.len() > 128 {
            return Err(ApiError::bad("commandId长度须为1至128字节"));
        }
        let seat = self.authenticate(id, token)?;
        let target = self.lookup(id)?;
        let mut current = target.game.lock().await;
        let duplicate: Option<(usize, u64, String, String)> = {
            let db = self.inner.db.lock().unwrap();
            db.query_row(
                "SELECT seat,expected_version,action,response FROM commands WHERE room_id=?1 AND command_id=?2",
                params![id, command.command_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .optional()?
        };
        if let Some((original_seat, original_version, original_action, response)) = duplicate {
            // Compare complete typed intent before returning a persisted receipt.
            // JSON key order and omitted/null optional fields normalize on decoding.
            let original_action: Action = serde_json::from_str(&original_action)?;
            if original_seat != seat
                || original_version != command.expected_version
                || original_action != command.action
            {
                return Err(ApiError {
                    status: StatusCode::CONFLICT,
                    error: "command_id_conflict".into(),
                    message: "commandId已用于另一座位、版本或动作；请为新意图生成新commandId"
                        .into(),
                    view: None,
                });
            }
            return Ok(serde_json::from_str(&response)?);
        }
        if command.expected_version != current.version {
            return Err(ApiError {
                status: StatusCode::CONFLICT,
                error: "version_conflict".into(),
                message: "房间已更新，请使用最新版本".into(),
                view: Some(Box::new(current.view(seat))),
            });
        }
        let mut next = current.clone();
        next.apply(seat, command.action.clone())
            .map_err(ApiError::bad)?;
        let view = next.view(seat);
        let serialized = serde_json::to_string(&next)?;
        let response = serde_json::to_string(&view)?;
        let action = serde_json::to_string(&command.action)?;
        let entry = serde_json::to_string(&Journal::Command {
            seat,
            command: command.clone(),
        })?;
        {
            let mut db = self.inner.db.lock().unwrap();
            let tx = db.transaction()?;
            tx.execute(
                "UPDATE rooms SET state=?2 WHERE id=?1",
                params![id, serialized],
            )?;
            tx.execute("INSERT INTO commands(room_id,command_id,seat,expected_version,action,response) VALUES(?1,?2,?3,?4,?5,?6)",params![id,command.command_id,seat,command.expected_version,action,response])?;
            tx.execute(
                "INSERT INTO journal(room_id,version,entry) VALUES(?1,?2,?3)",
                params![id, next.version, entry],
            )?;
            tx.commit()?;
        }
        *current = next;
        target.sender.send_replace(Arc::new(current.clone()));
        Ok(view)
    }
    /// Offline audit, never an HTTP endpoint: reproduce state using fixed versions + seed + ordered journal.
    pub fn replay(&self, id: &str) -> Result<Game, ApiError> {
        let db = self.inner.db.lock().unwrap();
        replay_connection(&db, id)
    }
    /// Opens an existing database with SQLite read-only flags; does not migrate,
    /// change pragmas, write state, or load tokens into the room service.
    pub fn open_read_only(path: impl AsRef<FilePath>) -> Result<Self, ApiError> {
        let db = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        db.busy_timeout(Duration::from_secs(5))?;
        Ok(Self {
            inner: Arc::new(Inner {
                db: Mutex::new(db),
                rooms: Mutex::new(HashMap::new()),
            }),
        })
    }
    /// Consistent read snapshot, even while the normal service continues writing.
    pub fn audit_replay(&self, id: &str) -> Result<ReplayAudit, ApiError> {
        let mut db = self.inner.db.lock().unwrap();
        let tx = db.transaction()?;
        let replayed = replay_connection(&tx, id)?;
        let stored: String =
            tx.query_row("SELECT state FROM rooms WHERE id=?1", [id], |r| r.get(0))?;
        let persisted = Game::from_persisted(&stored).map_err(ApiError::internal)?;
        let canonical = serde_json::to_vec(&persisted)?;
        let replay_bytes = serde_json::to_vec(&replayed)?;
        let journal_entries =
            tx.query_row("SELECT COUNT(*) FROM journal WHERE room_id=?1", [id], |r| {
                r.get(0)
            })?;
        let result = ReplayAudit {
            room_id: id.into(),
            version: persisted.version,
            journal_entries,
            status: persisted.status.clone(),
            matches: canonical == replay_bytes,
            persisted_digest: format!("{:x}", Sha256::digest(&canonical)),
            replayed_digest: format!("{:x}", Sha256::digest(&replay_bytes)),
            versions: persisted.versions,
        };
        tx.commit()?;
        Ok(result)
    }
}
fn replay_connection(db: &Connection, id: &str) -> Result<Game, ApiError> {
    let initial: String =
        db.query_row("SELECT initial_state FROM rooms WHERE id=?1", [id], |r| {
            r.get(0)
        })?;
    let mut game = Game::from_persisted(&initial).map_err(ApiError::internal)?;
    if game.versions.rules != catalog::RULES_VERSION
        || game.versions.card_pool != catalog::POOL_VERSION
        || game.versions.engine != catalog::ENGINE_VERSION
    {
        return Err(ApiError::internal("回放固定版本与当前引擎不符"));
    }
    let mut statement =
        db.prepare("SELECT version,entry FROM journal WHERE room_id=?1 ORDER BY version")?;
    let mut rows = statement.query([id])?;
    while let Some(row) = rows.next()? {
        let version: u64 = row.get(0)?;
        let entry: String = row.get(1)?;
        match serde_json::from_str::<Journal>(&entry)? {
            Journal::Join { name, deck_id } => {
                game.join(name, deck_id).map_err(ApiError::internal)?;
            }
            Journal::JoinWithDeck { name, deck_draft } => {
                game.join_with_deck(name, deck_draft)
                    .map_err(ApiError::internal)?;
            }
            Journal::Command { seat, command } => {
                if command.expected_version != game.version {
                    return Err(ApiError::internal("回放版本断裂"));
                }
                game.apply(seat, command.action)
                    .map_err(ApiError::internal)?;
            }
        }
        if game.version != version {
            return Err(ApiError::internal("回放版本不符"));
        }
    }
    Ok(game)
}
fn bearer(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .filter(|s| !s.is_empty())
        .ok_or_else(ApiError::unauthorized)
}
async fn health() -> Json<serde_json::Value> {
    Json(
        serde_json::json!({"ok":true,"service":"hegemony-rust","engineVersion":catalog::ENGINE_VERSION}),
    )
}
async fn get_catalog() -> Json<catalog::Catalog> {
    Json(catalog::catalog().clone())
}
async fn room_catalog(
    State(store): State<Store>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<catalog::Catalog>, ApiError> {
    store.authenticate(&id, bearer(&headers)?)?;
    store.lookup(&id)?; // Native databases accept only the current kernel's rooms.
    Ok(Json(catalog::catalog().clone()))
}
async fn create_room(
    State(store): State<Store>,
    Json(request): Json<CreateRoom>,
) -> Result<Json<Session>, ApiError> {
    Ok(Json(store.create(request).await?))
}
async fn join_room(
    State(store): State<Store>,
    Json(request): Json<JoinRoom>,
) -> Result<Json<Session>, ApiError> {
    Ok(Json(store.join(request).await?))
}
async fn get_state(
    State(store): State<Store>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<View>, ApiError> {
    Ok(Json(store.state(&id, bearer(&headers)?).await?))
}
async fn post_command(
    State(store): State<Store>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(command): Json<Command>,
) -> Result<Json<View>, ApiError> {
    Ok(Json(store.command(&id, bearer(&headers)?, command).await?))
}
async fn events(
    State(store): State<Store>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ApiError> {
    let seat = store.authenticate(&id, bearer(&headers)?)?;
    let target = store.lookup(&id)?;
    let mut receiver = target.sender.subscribe();
    let stream = async_stream::stream! {loop {let current=receiver.borrow_and_update().clone();let view=current.view(seat);match serde_json::to_string(&view){Ok(json)=>yield Ok::<Event,Infallible>(Event::default().id(current.version.to_string()).data(json)),Err(_)=>break}if receiver.changed().await.is_err(){break}}};
    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keepalive"),
    ))
}
pub fn router(store: Store) -> Router {
    let directory = std::env::var("WEB_DIST").unwrap_or_else(|_| "web/dist".into());
    Router::new()
        .route("/api/health", get(health))
        .route("/api/catalog", get(get_catalog))
        .route("/api/rooms", post(create_room))
        .route("/api/rooms/join", post(join_room))
        .route("/api/rooms/{roomId}/state", get(get_state))
        .route("/api/rooms/{roomId}/catalog", get(room_catalog))
        .route("/api/rooms/{roomId}/events", get(events))
        .route("/api/rooms/{roomId}/commands", post(post_command))
        .layer(DefaultBodyLimit::max(32 * 1024))
        .fallback_service(
            ServeDir::new(&directory).fallback(ServeFile::new(format!("{directory}/index.html"))),
        )
        .with_state(store)
}
