//! A Batna Cloud on 127.0.0.1 for the sync engine's tests. Routing, HTTP
//! and WebSocket plumbing are small stand-ins for `crates/server/src/wasm`,
//! but the device registry, tokens and the sync hub are the server's own
//! code (`batna_server`), on in-memory SQLite. A fake Google answers the
//! code exchange. Nothing leaves the machine.
//!
//! `crates/server/tests/e2e.mjs` checks the real Worker; this one runs in
//! every `cargo test`, and can turn the network off, close sockets and set
//! the server's clock to test what a phone on a train sees.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use batna_protocol::cloud::{
    ApiError, DeviceList, ErrorCode, GoogleSignIn, PasswordLogin, PasswordSignIn, RefreshRequest, Session,
    SetPassword, normalize_username, valid_username,
};
use batna_protocol::sync::{ServerMsg, SyncErrorCode};
use batna_server::devices::{Devices, RefreshError};
use batna_server::hub::{Conn, Hub};
use batna_server::login::{self as pw, AccountLogin, Checked, Login};
use batna_server::sql::{Row, Sql, SqlError, Value};
use batna_server::tokens::{self, TokenKey};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::protocol::CloseFrame;

/// The only Google account on the allowlist.
pub const EMAIL: &str = "mohamed@example.com";
/// What the fake Google signs a user in as when the code is this.
pub const CODE_OK: &str = "code-ok";
pub const CODE_STRANGER: &str = "code-stranger";

/// The ID token the fake Google hands out for `email`.
pub fn id_token(email: &str) -> String {
    format!("idtoken:{email}")
}

pub struct FakeCloud {
    pub url: String,
    state: Arc<Mutex<State>>,
}

struct Socket {
    conn_id: String,
    device: String,
    ready: bool,
    tx: UnboundedSender<Out>,
}

enum Out {
    Text(String),
    Binary,
    Close(u16),
    /// Ends the connection without a close frame.
    Cut,
}

struct State {
    db: MemDb,
    /// Each username's own store (the Worker's `LOGIN` Durable Objects).
    logins: HashMap<String, MemDb>,
    key: TokenKey,
    sockets: Vec<Socket>,
    next_conn: u64,
    /// Connections are dropped unanswered, like a phone without signal.
    offline: bool,
    /// Added to the real clock: negative means the server's clock is behind.
    skew_ms: i64,
    /// The hub only speaks a newer protocol.
    outdated: bool,
    /// The next N API calls (not the socket) answer 500.
    fail_next: u32,
    /// The next N socket upgrades answer 401 although the token is fine.
    refuse_upgrades: u32,
    /// Requests to this path drop the line, the next N times.
    drops: Option<(String, u32)>,
    /// The next N signed-in API calls answer 401 although the token is fine.
    unauthorized: u32,
    /// How many times a refresh token was used.
    refreshes: u32,
}

impl State {
    fn now(&self) -> u64 {
        (super::engine::now_ms() as i64 + self.skew_ms) as u64
    }
}

impl FakeCloud {
    pub fn start() -> Self {
        let db = MemDb(rusqlite::Connection::open_in_memory().unwrap());
        batna_server::account::open(&db, |w| panic!("{w}")).unwrap();
        let state = Arc::new(Mutex::new(State {
            db,
            logins: HashMap::new(),
            key: TokenKey::from_seed([7; 32]),
            sockets: Vec::new(),
            next_conn: 0,
            offline: false,
            skew_ms: 0,
            outdated: false,
            fail_next: 0,
            refuse_upgrades: 0,
            drops: None,
            unauthorized: 0,
            refreshes: 0,
        }));
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let shared = state.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            runtime.block_on(async move {
                let listener = TcpListener::from_std(listener).unwrap();
                while let Ok((stream, _)) = listener.accept().await {
                    tokio::spawn(serve(stream, shared.clone()));
                }
            });
        });
        Self { url, state }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap()
    }

    pub fn set_offline(&self, offline: bool) {
        let mut s = self.lock();
        s.offline = offline;
        if offline {
            // Open sockets go too, as when the train enters a tunnel.
            for socket in s.sockets.drain(..) {
                socket.tx.send(Out::Close(1001)).ok();
            }
        }
    }

    pub fn set_skew_ms(&self, skew_ms: i64) {
        self.lock().skew_ms = skew_ms;
    }

    pub fn set_outdated(&self, outdated: bool) {
        self.lock().outdated = outdated;
    }

    pub fn fail_next(&self, n: u32) {
        self.lock().fail_next = n;
    }

    pub fn refuse_upgrades(&self, n: u32) {
        self.lock().refuse_upgrades = n;
    }

    /// The next `n` requests to `path` get no answer at all.
    pub fn drop_next(&self, path: &str, n: u32) {
        self.lock().drops = Some((path.into(), n));
    }

    pub fn unauthorized_next(&self, n: u32) {
        self.lock().unauthorized = n;
    }

    /// Sends a binary frame to `device` (the app ignores those).
    pub fn send_binary(&self, device: &str) {
        for s in self.lock().sockets.iter().filter(|s| s.device == device) {
            s.tx.send(Out::Binary).ok();
        }
    }

    /// Cuts `device`'s sockets without a close frame.
    pub fn cut(&self, device: &str) {
        let mut s = self.lock();
        s.sockets.retain(|sock| {
            let mine = sock.device == device;
            if mine {
                sock.tx.send(Out::Cut).ok();
            }
            !mine
        });
    }

    pub fn refreshes(&self) -> u32 {
        self.lock().refreshes
    }

    /// Open sync sockets.
    pub fn sockets(&self) -> usize {
        self.lock().sockets.len()
    }

    /// Sends a raw frame to every socket of `device` (a hub error, junk).
    pub fn send_raw(&self, device: &str, text: &str) {
        for s in self.lock().sockets.iter().filter(|s| s.device == device) {
            s.tx.send(Out::Text(text.into())).ok();
        }
    }

    /// Closes every socket of `device` with `code`.
    pub fn close(&self, device: &str, code: u16) {
        let mut s = self.lock();
        s.sockets.retain(|sock| {
            if sock.device == device {
                sock.tx.send(Out::Close(code)).ok();
                false
            } else {
                true
            }
        });
    }

    /// A ready-made session for `device`, as a sign-in returns it.
    pub fn session_for(&self, device: &batna_protocol::cloud::DeviceInfo) -> Session {
        let mut s = self.lock();
        sign_in(&mut s, EMAIL, device.clone()).unwrap()
    }

    /// The hub's current payload of a row.
    /// Another account already holds `username`.
    pub fn take_username(&self, username: &str) {
        let mut s = self.lock();
        let now = s.now();
        let db = login_db(&mut s, username);
        assert!(Login::new(db).unwrap().set("someone-else", "someone@example.com", "00", "x", now).unwrap());
    }

    pub fn payload(&self, table: &str, row_id: &str) -> Option<serde_json::Value> {
        let s = self.lock();
        Hub::open(&s.db).current_payload(table, row_id).unwrap()
    }
}

fn account_id() -> String {
    tokens::account_id_for_google_sub(EMAIL)
}

fn login_db<'a>(s: &'a mut State, username: &str) -> &'a MemDb {
    s.logins
        .entry(username.to_owned())
        .or_insert_with(|| MemDb(rusqlite::Connection::open_in_memory().unwrap()))
}

fn sign_in(s: &mut State, email: &str, device: batna_protocol::cloud::DeviceInfo) -> Result<Session, Reply> {
    if email != EMAIL {
        return Err(error(ErrorCode::NotAllowed, "this Google account is not on Batna's allowlist"));
    }
    Ok(session(s, account_id(), email, device))
}

/// Like the Worker's `sign_in_password`.
fn sign_in_password(s: &mut State, body: &str) -> Reply {
    let Ok(req) = serde_json::from_str::<PasswordSignIn>(body) else {
        return error(ErrorCode::InvalidRequest, "bad body");
    };
    let username = normalize_username(&req.username);
    let Some(key) = pw::parse_key(&req.password_key).filter(|_| valid_username(&username)) else {
        return error(ErrorCode::WrongPassword, "username or password is wrong");
    };
    let now = s.now();
    match Login::new(login_db(s, &username)).unwrap().check(&key, now).unwrap() {
        Checked::Ok { account_id, email } => ok(&session(s, account_id, &email, req.device)),
        Checked::Wrong => error(ErrorCode::WrongPassword, "username or password is wrong"),
        Checked::Locked { retry_after_ms } => (
            429,
            serde_json::to_string(&ApiError {
                code: ErrorCode::TooManyTries,
                message: "too many wrong passwords for this username".into(),
                retry_after_ms: Some(retry_after_ms),
            })
            .unwrap(),
        ),
    }
}

/// Like the Worker's `manage_login`, for the one account.
fn manage_login(s: &mut State, method: &str, account: &str, body: &str) -> Reply {
    let now = s.now();
    let old = AccountLogin::open(&s.db).username().unwrap();
    let username = match method {
        "GET" => old,
        "POST" => {
            let Ok(set) = serde_json::from_str::<SetPassword>(body) else {
                return error(ErrorCode::InvalidRequest, "bad body");
            };
            let username = normalize_username(&set.username);
            let Some(key) = pw::parse_key(&set.password_key).filter(|_| valid_username(&username)) else {
                return error(ErrorCode::InvalidRequest, "bad username or key");
            };
            let salt = "5a17";
            let verifier = pw::verifier(salt, &key);
            if !Login::new(login_db(s, &username)).unwrap().set(account, EMAIL, salt, &verifier, now).unwrap()
            {
                return error(ErrorCode::UsernameTaken, "another account has this username");
            }
            if let Some(old) = old.filter(|o| *o != username) {
                Login::new(login_db(s, &old)).unwrap().clear(account).unwrap();
            }
            Some(username)
        }
        _ => {
            if let Some(old) = &old {
                Login::new(login_db(s, old)).unwrap().clear(account).unwrap();
            }
            None
        }
    };
    AccountLogin::open(&s.db).set_username(username.as_deref()).unwrap();
    ok(&PasswordLogin { username })
}

fn session(
    s: &mut State,
    account: String,
    email: &str,
    device: batna_protocol::cloud::DeviceInfo,
) -> Session {
    let refresh = tokens::new_refresh(&account, &device.device_id, tokens::random_bytes().unwrap());
    let now = s.now();
    let expires = Devices::open(&s.db).register(&device, email, &refresh.hash, now).unwrap();
    let (access_token, access_expires_at_ms) = tokens::issue_access(&s.key, &account, &device.device_id, now);
    Session {
        account_id: account,
        device_id: device.device_id,
        email: Some(email.into()),
        access_token,
        access_expires_at_ms,
        refresh_token: refresh.token,
        refresh_expires_at_ms: expires,
    }
}

type Reply = (u16, String);

fn error(code: ErrorCode, message: &str) -> Reply {
    let status = match code {
        ErrorCode::InvalidRequest => 400,
        ErrorCode::Unauthorized | ErrorCode::DeviceRevoked => 401,
        ErrorCode::NotAllowed => 403,
        ErrorCode::NotFound => 404,
        ErrorCode::WrongPassword => 401,
        ErrorCode::TooManyTries => 429,
        ErrorCode::UsernameTaken => 409,
        ErrorCode::Internal | ErrorCode::Other => 500,
    };
    (
        status,
        serde_json::to_string(&ApiError { code, message: message.into(), retry_after_ms: None }).unwrap(),
    )
}

fn ok(body: &impl serde::Serialize) -> Reply {
    (200, serde_json::to_string(body).unwrap())
}

/// Tells the device's sockets it was signed out, then closes them (4001).
fn close_device(s: &mut State, device: &str) {
    let revoked = serde_json::to_string(&ServerMsg::Revoked).unwrap();
    s.sockets.retain(|sock| {
        if sock.device == device {
            sock.tx.send(Out::Text(revoked.clone())).ok();
            sock.tx.send(Out::Close(4001)).ok();
            false
        } else {
            true
        }
    });
}

async fn serve(mut stream: TcpStream, state: Arc<Mutex<State>>) {
    if state.lock().unwrap().offline {
        return;
    }
    let mut first = [0u8; 12];
    let n = stream.peek(&mut first).await.unwrap_or(0);
    if first[..n].starts_with(b"GET /v1/sync") {
        socket(stream, state).await;
        return;
    }
    let Some((method, path, headers, body)) = read_request(&mut stream).await else { return };
    let reply = {
        let mut s = state.lock().unwrap();
        if let Some((drop, n)) = &mut s.drops
            && *drop == path
            && *n > 0
        {
            *n -= 1;
            return;
        }
        if s.fail_next > 0 && path.starts_with("/v1/") {
            s.fail_next -= 1;
            (502, "<html>Bad gateway</html>".into())
        } else {
            route(&mut s, &method, &path, &headers, &body)
        }
    };
    let text = format!(
        "HTTP/1.1 {} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
        reply.0,
        reply.1.len(),
        reply.1
    );
    let _ = stream.write_all(text.as_bytes()).await;
    let _ = stream.shutdown().await;
}

async fn read_request(stream: &mut TcpStream) -> Option<(String, String, Vec<(String, String)>, String)> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let (method, path) = (first.next()?.to_string(), first.next()?.to_string());
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    let len: usize =
        headers.iter().find(|(k, _)| k == "content-length").and_then(|(_, v)| v.parse().ok()).unwrap_or(0);
    while buf.len() < head_end + len {
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let body = String::from_utf8_lossy(&buf[head_end..]).into_owned();
    Some((method, path, headers, body))
}

fn bearer(headers: &[(String, String)]) -> Option<&str> {
    headers.iter().find(|(k, _)| k == "authorization").and_then(|(_, v)| v.strip_prefix("Bearer "))
}

fn route(s: &mut State, method: &str, path: &str, headers: &[(String, String)], body: &str) -> Reply {
    let now = s.now();
    match (method, path) {
        // Google's token endpoint: a form with the code from the redirect.
        ("POST", "/google/token") => {
            let code = body.split('&').find_map(|p| p.strip_prefix("code=")).unwrap_or_default();
            let email = match code {
                CODE_OK => EMAIL,
                CODE_STRANGER => "stranger@example.com",
                _ => return (400, r#"{"error":"invalid_grant"}"#.into()),
            };
            ok(&serde_json::json!({ "id_token": id_token(email) }))
        }
        ("POST", "/v1/auth/google") => {
            let Ok(req) = serde_json::from_str::<GoogleSignIn>(body) else {
                return error(ErrorCode::InvalidRequest, "bad body");
            };
            let Some(email) = req.id_token.strip_prefix("idtoken:") else {
                return error(ErrorCode::Unauthorized, "Google ID token: bad signature");
            };
            match sign_in(s, email, req.device) {
                Ok(session) => ok(&session),
                Err(reply) => reply,
            }
        }
        ("POST", "/v1/auth/password") => sign_in_password(s, body),
        ("POST", "/v1/auth/refresh") => {
            s.refreshes += 1;
            let Ok(req) = serde_json::from_str::<RefreshRequest>(body) else {
                return error(ErrorCode::InvalidRequest, "bad body");
            };
            let Some(parsed) = tokens::parse_refresh(&req.refresh_token) else {
                return error(ErrorCode::Unauthorized, "malformed refresh token");
            };
            let fresh =
                tokens::new_refresh(&parsed.account_id, &parsed.device, tokens::random_bytes().unwrap());
            let refreshed = Devices::open(&s.db).refresh(&parsed.device, &parsed.hash, &fresh.hash, now);
            match refreshed {
                Ok(r) => {
                    let (access_token, access_expires_at_ms) =
                        tokens::issue_access(&s.key, &parsed.account_id, &parsed.device, now);
                    ok(&Session {
                        account_id: parsed.account_id,
                        device_id: parsed.device,
                        email: Some(r.email),
                        access_token,
                        access_expires_at_ms,
                        refresh_token: fresh.token,
                        refresh_expires_at_ms: r.expires_ms,
                    })
                }
                Err(e @ (RefreshError::Revoked | RefreshError::Reused)) => {
                    close_device(s, &parsed.device);
                    error(ErrorCode::DeviceRevoked, &e.to_string())
                }
                Err(e) => error(ErrorCode::Unauthorized, &e.to_string()),
            }
        }
        _ => {
            let claims = match bearer(headers).map(|t| tokens::verify_access(&s.key.public(), t, now)) {
                Some(Ok(c)) if s.unauthorized == 0 => c,
                _ => {
                    s.unauthorized = s.unauthorized.saturating_sub(1);
                    return error(ErrorCode::Unauthorized, "missing or bad token");
                }
            };
            let devices = Devices::open(&s.db);
            let active = devices.is_active(&claims.dev).unwrap();
            match (method, path) {
                ("GET", "/v1/devices") if active => {
                    ok(&DeviceList { devices: devices.list(&claims.dev).unwrap() })
                }
                ("DELETE", p) if p.starts_with("/v1/devices/") => {
                    let id = &p["/v1/devices/".len()..];
                    if id != claims.dev && !active {
                        return error(ErrorCode::DeviceRevoked, "this device was signed out");
                    }
                    if devices.revoke(id, now).unwrap() {
                        close_device(s, id);
                        (200, String::new())
                    } else {
                        error(ErrorCode::NotFound, "no such device")
                    }
                }
                ("POST", "/v1/auth/signout") => {
                    devices.revoke(&claims.dev, now).unwrap();
                    close_device(s, &claims.dev);
                    (200, String::new())
                }
                (m @ ("GET" | "POST" | "DELETE"), "/v1/login") if active => {
                    manage_login(s, m, &claims.sub, body)
                }
                ("GET", "/v1/devices") => error(ErrorCode::DeviceRevoked, "this device was signed out"),
                _ => error(ErrorCode::NotFound, "not found"),
            }
        }
    }
}

async fn socket(stream: TcpStream, state: Arc<Mutex<State>>) {
    let mut device = None;
    // tungstenite's handshake callback fixes this signature.
    #[allow(clippy::result_large_err)]
    let check = |req: &Request, res: Response| -> Result<Response, ErrorResponse> {
        let mut s = state.lock().unwrap();
        let token = req.headers().get("authorization").and_then(|v| v.to_str().ok());
        let claims = token
            .and_then(|t| t.strip_prefix("Bearer "))
            .and_then(|t| tokens::verify_access(&s.key.public(), t, s.now()).ok());
        let refused = s.refuse_upgrades > 0;
        s.refuse_upgrades = s.refuse_upgrades.saturating_sub(1);
        match claims {
            Some(c) if !refused && Devices::open(&s.db).is_active(&c.dev).unwrap() => {
                device = Some(c.dev);
                Ok(res)
            }
            _ => {
                let mut no = ErrorResponse::new(Some("unauthorized".into()));
                *no.status_mut() = tokio_tungstenite::tungstenite::http::StatusCode::UNAUTHORIZED;
                Err(no)
            }
        }
    };
    let Ok(ws) = tokio_tungstenite::accept_hdr_async(stream, check).await else { return };
    let Some(device) = device else { return };
    let (mut sink, mut source) = ws.split();
    let (tx, mut rx) = unbounded_channel();
    let conn_id = {
        let mut s = state.lock().unwrap();
        s.next_conn += 1;
        let conn_id = format!("c{}", s.next_conn);
        s.sockets.push(Socket { conn_id: conn_id.clone(), device: device.clone(), ready: false, tx });
        conn_id
    };
    let mut conn = Conn { device, conn_id: conn_id.clone(), ready: false };
    loop {
        tokio::select! {
            out = rx.recv() => match out {
                Some(Out::Text(t)) => { if sink.send(Message::Text(t.into())).await.is_err() { break } }
                Some(Out::Binary) => { let _ = sink.send(Message::Binary(vec![1, 2].into())).await; }
                Some(Out::Cut) => return,
                Some(Out::Close(code)) => {
                    let frame = CloseFrame { code: code.into(), reason: "bye".into() };
                    let _ = sink.send(Message::Close(Some(frame))).await;
                    break;
                }
                None => break,
            },
            msg = source.next() => match msg {
                Some(Ok(Message::Text(text))) => {
                    let mut s = state.lock().unwrap();
                    for (to, reply) in handle(&mut s, &mut conn, &text) {
                        match to {
                            Some(c) => s.sockets.iter().filter(|x| x.conn_id == c).for_each(|x| { x.tx.send(Out::Text(reply.clone())).ok(); }),
                            None => s.sockets.iter().filter(|x| x.ready && x.conn_id != conn.conn_id).for_each(|x| { x.tx.send(Out::Text(reply.clone())).ok(); }),
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                Some(Ok(_)) => {}
            },
        }
    }
    state.lock().unwrap().sockets.retain(|s| s.conn_id != conn_id);
}

/// One client message through the server's hub. `Some(conn)` replies go to
/// the sender, `None` ones to every other ready socket.
fn handle(s: &mut State, conn: &mut Conn, text: &str) -> Vec<(Option<String>, String)> {
    let me = Some(conn.conn_id.clone());
    let msg = |m: &ServerMsg| serde_json::to_string(m).unwrap();
    if s.outdated && text.contains("\"hello\"") {
        let m =
            ServerMsg::Error { code: SyncErrorCode::UnsupportedProtocol, message: "server speaks 99".into() };
        return vec![(me, msg(&m))];
    }
    let now = s.now();
    let devices = Devices::open(&s.db);
    if !devices.is_active(&conn.device).unwrap() {
        return vec![(me, msg(&ServerMsg::Revoked))];
    }
    let outcome = Hub::open(&s.db).handle(conn, text, now).unwrap();
    devices.touch(&conn.device, now, outcome.synced).unwrap();
    if let Some(sock) = s.sockets.iter_mut().find(|x| x.conn_id == conn.conn_id) {
        sock.ready = conn.ready;
    }
    let mut out: Vec<_> = outcome.replies.iter().map(|r| (me.clone(), msg(r))).collect();
    if !outcome.broadcast.is_empty() {
        out.push((None, msg(&ServerMsg::Ops { ops: outcome.broadcast, more: false })));
    }
    out
}

/// In-memory SQLite standing in for Durable Object storage (as in the
/// server's own tests).
struct MemDb(rusqlite::Connection);

impl Sql for MemDb {
    fn exec(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>, SqlError> {
        use rusqlite::types::{Value as V, ValueRef};
        let err = |e: rusqlite::Error| SqlError(e.to_string());
        let mut stmt = self.0.prepare(sql).map_err(err)?;
        let params: Vec<V> = params
            .iter()
            .map(|p| match p {
                Value::Null => V::Null,
                Value::Int(i) => V::Integer(*i),
                Value::Real(f) => V::Real(*f),
                Value::Text(s) => V::Text(s.clone()),
            })
            .collect();
        let n = stmt.column_count();
        let mut rows = stmt.query(rusqlite::params_from_iter(params)).map_err(err)?;
        let mut out = Vec::new();
        while let Some(row) = rows.next().map_err(err)? {
            let mut r = Vec::with_capacity(n);
            for i in 0..n {
                r.push(match row.get_ref(i).map_err(err)? {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(i) => Value::Int(i),
                    ValueRef::Real(f) => Value::Real(f),
                    ValueRef::Text(t) => Value::Text(String::from_utf8_lossy(t).into_owned()),
                    ValueRef::Blob(_) => return Err(SqlError("blobs are not used".into())),
                });
            }
            out.push(r);
        }
        Ok(out)
    }
}
