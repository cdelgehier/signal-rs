use crate::models::{Channel, Message, PairingStatus, ReceiptStatus};
use futures::channel::oneshot as futures_oneshot;
use futures::StreamExt;
use presage::libsignal_service::configuration::SignalServers;
use presage::libsignal_service::content::ContentBody;
use presage::libsignal_service::proto::DataMessage;
use presage::libsignal_service::protocol::{Aci, ServiceId};
use presage::manager::Registered;
use presage::model::identity::OnNewIdentity;
use presage::model::messages::Received;
use presage::store::{ContentsStore, Thread};
use presage::Manager;
use presage_store_sqlite::SqliteStore;
use std::path::PathBuf;
use tauri::Emitter;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{debug, error, info};
use url::Url;
use uuid::Uuid;

pub type SignalManager = Manager<SqliteStore, Registered>;

// ---------------------------------------------------------------------------
// Actor messages
// ---------------------------------------------------------------------------

enum Request {
    GenerateQr(oneshot::Sender<Result<String, String>>),
    GetPairingStatus(oneshot::Sender<PairingStatus>),
    GetChannels(oneshot::Sender<Result<Vec<Channel>, String>>),
    GetMessages(String, oneshot::Sender<Result<Vec<Message>, String>>),
    SendMessage(String, String, oneshot::Sender<Result<(), String>>),
}

// ---------------------------------------------------------------------------
// Public handle — Send + Sync, stored in Tauri State
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ManagerHandle {
    tx: mpsc::Sender<Request>,
    pub pairing_status_rx: watch::Receiver<PairingStatus>,
}

impl ManagerHandle {
    pub async fn generate_qr(&self) -> Result<String, String> {
        let (tx, rx) = oneshot::channel();
        self.send(Request::GenerateQr(tx)).await?;
        rx.await.map_err(|_| "Actor stopped".to_string())?
    }

    pub async fn get_pairing_status(&self) -> Result<PairingStatus, String> {
        let (tx, rx) = oneshot::channel();
        self.send(Request::GetPairingStatus(tx)).await?;
        rx.await.map_err(|_| "Actor stopped".to_string())
    }

    pub async fn get_channels(&self) -> Result<Vec<Channel>, String> {
        let (tx, rx) = oneshot::channel();
        self.send(Request::GetChannels(tx)).await?;
        rx.await.map_err(|_| "Actor stopped".to_string())?
    }

    pub async fn get_messages(&self, channel_id: String) -> Result<Vec<Message>, String> {
        let (tx, rx) = oneshot::channel();
        self.send(Request::GetMessages(channel_id, tx)).await?;
        rx.await.map_err(|_| "Actor stopped".to_string())?
    }

    pub async fn send_message(&self, channel_id: String, text: String) -> Result<(), String> {
        let (tx, rx) = oneshot::channel();
        self.send(Request::SendMessage(channel_id, text, tx))
            .await?;
        rx.await.map_err(|_| "Actor stopped".to_string())?
    }

    async fn send(&self, req: Request) -> Result<(), String> {
        self.tx
            .send(req)
            .await
            .map_err(|_| "Manager actor stopped".to_string())
    }
}

// ---------------------------------------------------------------------------
// Spawn actor on a dedicated thread with single-thread runtime + LocalSet
// ---------------------------------------------------------------------------

pub fn spawn_actor(app_handle: tauri::AppHandle) -> ManagerHandle {
    let (tx, rx) = mpsc::channel::<Request>(32);
    let (status_tx, status_rx) = watch::channel(PairingStatus::Pending);

    // presage's StreamState struct is large — needs a bigger stack than the default 8MB
    std::thread::Builder::new()
        .name("signal-actor".into())
        .stack_size(32 * 1024 * 1024) // 32 MB
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to build single-thread runtime");

            let local = tokio::task::LocalSet::new();
            rt.block_on(local.run_until(actor_loop(rx, status_tx, app_handle)));
        })
        .expect("failed to spawn signal actor thread");

    ManagerHandle {
        tx,
        pairing_status_rx: status_rx,
    }
}

// ---------------------------------------------------------------------------
// Actor loop — no recursion, plain loop
// ---------------------------------------------------------------------------

async fn actor_loop(
    mut rx: mpsc::Receiver<Request>,
    status_tx: watch::Sender<PairingStatus>,
    app_handle: tauri::AppHandle,
) {
    let mut manager: Option<SignalManager> = None;

    match open_store().await {
        Ok(Some(mgr)) => {
            info!("Existing registration loaded");
            manager = Some(mgr);
            let _ = status_tx.send(PairingStatus::Linked);
            // Give the frontend time to mount and register listeners, then push contacts
            let ah = app_handle.clone();
            tokio::task::spawn_local(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                ah.emit("contacts-updated", ()).ok();
            });
        }
        Ok(None) => debug!("No existing registration"),
        Err(e) => error!("open_store: {e}"),
    }

    loop {
        match manager.as_mut() {
            // Registered: run receive loop + handle requests via select!
            Some(_) => {
                let mgr = manager.as_mut().unwrap();
                let should_continue = receive_loop(mgr, &mut rx, &status_tx, &app_handle).await;

                if !should_continue {
                    // Either deassociated or rx closed.
                    // Check if status was reset to Pending (= deassociation).
                    if *status_tx.borrow() == PairingStatus::Pending {
                        info!("Device deassociated — wiping store and returning to QR flow");
                        // Wipe the SQLite DB so next open_store() returns None
                        let path = store_path();
                        for ext in &["", "-shm", "-wal"] {
                            std::fs::remove_file(format!("{path}{ext}")).ok();
                        }
                        manager = None;
                        // Continue the outer loop → will go to the None branch and wait for QR
                        continue;
                    }
                    break; // genuine shutdown (rx closed)
                }
                info!("Reconnecting in 3s…");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
            // Not registered: handle requests until QR pairing completes
            None => {
                let Some(req) = rx.recv().await else { break };
                dispatch_unlinked(req, &mut manager, &status_tx).await;
                if manager.is_some() {
                    let _ = status_tx.send(PairingStatus::Linked);
                    app_handle.emit("contacts-updated", ()).ok();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Receive loop — interleaves Signal stream with Tauri commands via select!
//
// presage-cli proves that the stream returned by receive_messages() does NOT
// hold a borrow on &mut manager: the cli passes &mut manager to
// process_incoming_message() inside the same while loop.
// ---------------------------------------------------------------------------

/// Return values for receive_loop:
/// - `true`  = reconnect (temporary failure)
/// - `false` = shutdown or deassociation (caller checks `PairingStatus`)
async fn receive_loop(
    mgr: &mut SignalManager,
    rx: &mut mpsc::Receiver<Request>,
    status_tx: &watch::Sender<PairingStatus>,
    app_handle: &tauri::AppHandle,
) -> bool {
    let stream = match mgr.receive_messages().await {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("{e}");
            // 401 / Unauthorized / NotRegistered → device was deassociated
            if msg.contains("401")
                || msg.contains("nauthorized")
                || msg.contains("NotRegistered")
                || msg.contains("not registered")
                || msg.contains("websocket upgrade failed")
                || msg.contains("upgrade failed")
            {
                error!("Device deassociated: {e}");
                let _ = status_tx.send(PairingStatus::Pending);
                app_handle.emit("deassociated", ()).ok();
                return false; // signal caller to reset
            }
            error!("receive_messages: {e}");
            return true; // reconnect
        }
    };
    futures::pin_mut!(stream);

    loop {
        tokio::select! {
            item = stream.next() => {
                match item {
                    Some(Received::QueueEmpty) => {
                        info!("Initial sync complete");
                        app_handle.emit("contacts-updated", ()).ok();
                    }
                    Some(Received::Contacts) => {
                        info!("Contacts synced");
                        app_handle.emit("contacts-updated", ()).ok();
                    }
                    Some(Received::Content(content)) => {
                        if let Some((channel_id, msg)) = content_to_message(&content) {
                            app_handle.emit("message-received", (&channel_id, msg)).ok();
                        }
                    }
                    None => return true, // stream ended → reconnect
                }
            }
            req = rx.recv() => {
                let Some(req) = req else { return false }; // rx closed → shutdown
                dispatch_linked(req, mgr, status_tx).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Dispatch: device NOT yet registered
// ---------------------------------------------------------------------------

async fn dispatch_unlinked(
    req: Request,
    manager: &mut Option<SignalManager>,
    status_tx: &watch::Sender<PairingStatus>,
) {
    match req {
        Request::GenerateQr(reply) => {
            let _ = reply.send(handle_generate_qr(manager, status_tx).await);
        }
        Request::GetPairingStatus(reply) => {
            let _ = reply.send(status_tx.borrow().clone());
        }
        Request::GetChannels(reply) => {
            let _ = reply.send(Err("Not linked to Signal".to_string()));
        }
        Request::GetMessages(_, reply) => {
            let _ = reply.send(Err("Not linked to Signal".to_string()));
        }
        Request::SendMessage(_, _, reply) => {
            let _ = reply.send(Err("Not linked to Signal".to_string()));
        }
    }
}

// ---------------------------------------------------------------------------
// Dispatch: device IS registered (mgr = &mut SignalManager)
// ---------------------------------------------------------------------------

async fn dispatch_linked(
    req: Request,
    mgr: &mut SignalManager,
    status_tx: &watch::Sender<PairingStatus>,
) {
    match req {
        Request::GenerateQr(reply) => {
            let _ = reply.send(Ok(String::new())); // already linked
        }
        Request::GetPairingStatus(reply) => {
            let _ = reply.send(status_tx.borrow().clone());
        }
        Request::GetChannels(reply) => {
            let result = fetch_channels(mgr).await.map_err(|e| e.to_string());
            let _ = reply.send(result);
        }
        Request::GetMessages(channel_id, reply) => {
            let result = fetch_messages(mgr, &channel_id, 50)
                .await
                .map_err(|e| e.to_string());
            let _ = reply.send(result);
        }
        Request::SendMessage(channel_id, text, reply) => {
            let result = send_text(mgr, &channel_id, text)
                .await
                .map_err(|e| e.to_string());
            let _ = reply.send(result);
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn store_path() -> String {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("signal-rs");
    std::fs::create_dir_all(&base).ok();
    base.join("signal.db")
        .to_str()
        .expect("non-UTF8 store path")
        .to_owned()
}

async fn open_store() -> anyhow::Result<Option<SignalManager>> {
    let path = store_path();
    let store = SqliteStore::open_with_passphrase(&path, None, OnNewIdentity::Trust).await?;
    match Manager::load_registered(store).await {
        Ok(m) => Ok(Some(m)),
        Err(e) => {
            debug!("load_registered: {e}");
            Ok(None)
        }
    }
}

async fn handle_generate_qr(
    manager: &mut Option<SignalManager>,
    status_tx: &watch::Sender<PairingStatus>,
) -> Result<String, String> {
    if manager.is_some() {
        return Ok(String::new());
    }
    let path = store_path();
    let store = SqliteStore::open_with_passphrase(&path, None, OnNewIdentity::Trust)
        .await
        .map_err(|e| format!("open_store: {e}"))?;

    let (link_tx, link_rx) = futures_oneshot::channel::<Url>();
    let status_tx_clone = status_tx.clone();

    tokio::task::spawn_local(async move {
        match Manager::link_secondary_device(
            store,
            SignalServers::Production,
            "Signal RS".into(),
            link_tx,
        )
        .await
        {
            Ok(_) => {
                let _ = status_tx_clone.send(PairingStatus::Linked);
            }
            Err(e) => error!("link_secondary_device: {e}"),
        }
    });

    let url: Url = link_rx
        .await
        .map_err(|_| "Provisioning channel closed".to_string())?;

    url_to_qr_png_base64(url.as_str()).map_err(|e| format!("QR: {e}"))
}

pub fn url_to_qr_png_base64(url: &str) -> anyhow::Result<String> {
    use base64::Engine;
    use image::ImageFormat;
    use qrcode::QrCode;
    use std::io::Cursor;

    let code = QrCode::new(url.as_bytes())?;
    let img = code.render::<image::Luma<u8>>().build();
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(buf.into_inner()))
}

fn content_to_message(
    content: &presage::libsignal_service::content::Content,
) -> Option<(String, Message)> {
    match &content.body {
        ContentBody::DataMessage(dm) => {
            let text = dm.body.clone()?;
            let sender_uuid = content.metadata.sender.raw_uuid();
            let sender_id = sender_uuid.to_string();
            // channel = sender (incoming DM)
            Some((
                sender_id.clone(),
                Message {
                    id: content.metadata.timestamp,
                    sender_id: sender_id.clone(),
                    sender_name: sender_id,
                    text: Some(text),
                    timestamp: content.metadata.timestamp as i64,
                    is_outgoing: false,
                    receipt: None,
                },
            ))
        }
        ContentBody::SynchronizeMessage(sync) => {
            let sent = sync.sent.as_ref()?;
            let text = sent.message.as_ref().and_then(|dm| dm.body.clone())?;
            let sender_uuid = content.metadata.sender.raw_uuid();
            // destination = who the message was sent TO (the channel)
            let channel_id = sent
                .destination_service_id
                .as_deref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| sender_uuid.to_string());
            Some((
                channel_id.clone(),
                Message {
                    id: content.metadata.timestamp,
                    sender_id: sender_uuid.to_string(),
                    sender_name: "Me".to_string(),
                    text: Some(text),
                    timestamp: content.metadata.timestamp as i64,
                    is_outgoing: true,
                    receipt: Some(ReceiptStatus::Sent),
                },
            ))
        }
        _ => None,
    }
}

async fn fetch_channels(mgr: &mut SignalManager) -> anyhow::Result<Vec<Channel>> {
    let mut channels = Vec::new();
    let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

    // 1. Named contacts from the contacts store
    for result in mgr.store().contacts().await? {
        let contact = match result {
            Ok(c) => c,
            Err(e) => {
                error!("contact: {e}");
                continue;
            }
        };
        let id = contact.uuid.to_string();
        seen_ids.insert(id.clone());
        let name = if contact.name.is_empty() {
            id.clone()
        } else {
            contact.name
        };

        // Fetch the most recent message for this thread
        let service_id: ServiceId = Aci::from(contact.uuid).into();
        let thread = Thread::Contact(service_id);
        let (last_message, last_message_time) = mgr
            .store()
            .messages(&thread, ..)
            .await
            .ok()
            .and_then(|mut iter| {
                // messages() returns in DESC order — first = most recent
                iter.next().and_then(|r| r.ok()).map(|content| {
                    let text = match &content.body {
                        ContentBody::DataMessage(dm) => dm.body.clone().or_else(|| {
                            if dm.attachments.is_empty() {
                                None
                            } else {
                                Some("📎 Attachment".into())
                            }
                        }),
                        ContentBody::SynchronizeMessage(s) => s
                            .sent
                            .as_ref()
                            .and_then(|s| s.message.as_ref())
                            .and_then(|dm| {
                                dm.body.clone().or_else(|| {
                                    if dm.attachments.is_empty() {
                                        None
                                    } else {
                                        Some("📎 Attachment".into())
                                    }
                                })
                            }),
                        _ => None,
                    };
                    (text, Some(content.metadata.timestamp as i64))
                })
            })
            .unwrap_or((None, None));

        channels.push(Channel {
            id,
            name,
            last_message,
            last_message_time,
            unread_count: 0,
            is_group: false,
        });
    }

    // 2. Groups from the groups store
    for result in mgr.store().groups().await? {
        let (master_key, group) = match result {
            Ok(g) => g,
            Err(e) => {
                error!("group: {e}");
                continue;
            }
        };
        let id = hex::encode(master_key);
        seen_ids.insert(id.clone());
        let name = if group.title.is_empty() {
            id.clone()
        } else {
            group.title
        };
        channels.push(Channel {
            id,
            name,
            last_message: None,
            last_message_time: None,
            unread_count: 0,
            is_group: true,
        });
    }

    // Sort by most recent message first
    channels.sort_by(|a, b| b.last_message_time.cmp(&a.last_message_time));
    Ok(channels)
}

async fn fetch_messages(
    mgr: &mut SignalManager,
    channel_id: &str,
    limit: usize,
) -> anyhow::Result<Vec<Message>> {
    let uuid = Uuid::parse_str(channel_id)?;
    let service_id: ServiceId = Aci::from(uuid).into();
    let thread = Thread::Contact(service_id);

    let mut messages: Vec<Message> = mgr
        .store()
        .messages(&thread, ..)
        .await?
        .take(limit)
        .filter_map(|r| {
            let c = r.map_err(|e| error!("msg: {e}")).ok()?;
            // SynchronizeMessage = sent from one of our own devices → outgoing
            let (text, is_outgoing) = match &c.body {
                ContentBody::DataMessage(dm) => (dm.body.clone(), false),
                ContentBody::SynchronizeMessage(s) => {
                    let text = s
                        .sent
                        .as_ref()
                        .and_then(|s| s.message.as_ref())
                        .and_then(|dm| dm.body.clone());
                    (text, true)
                }
                _ => return None,
            };
            let sender_id = c.metadata.sender.raw_uuid().to_string();
            Some(Message {
                id: c.metadata.timestamp,
                sender_id: sender_id.clone(),
                sender_name: if is_outgoing {
                    "Me".to_string()
                } else {
                    sender_id
                },
                text,
                timestamp: c.metadata.timestamp as i64,
                is_outgoing,
                receipt: Some(ReceiptStatus::Sent),
            })
        })
        .collect();

    messages.sort_by_key(|m| m.timestamp);
    Ok(messages)
}

async fn send_text(mgr: &mut SignalManager, channel_id: &str, text: String) -> anyhow::Result<()> {
    let aci = Aci::from(Uuid::parse_str(channel_id)?);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;
    mgr.send_message(
        aci,
        DataMessage {
            body: Some(text),
            timestamp: Some(ts),
            ..Default::default()
        },
        ts,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
}
