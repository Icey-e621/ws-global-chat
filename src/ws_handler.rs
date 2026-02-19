use futures_util::{SinkExt, StreamExt as _};
use tokio::sync::{broadcast, RwLock};
use warp::filters::ws::{Message, WebSocket};
use crate::db::ChatMessage;
use std::sync::Arc;
use std::collections::HashSet;

pub async fn handle_connection(
    pool: sqlx::MySqlPool,
    ws: WebSocket,
    tx: broadcast::Sender<String>,
    session_cache: Arc<RwLock<HashSet<String>>>,
) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut rx = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    if let Ok(mut chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                        let mut is_valid = false;
                        if let Some(ref token) = chat_msg.session_id {
                            // 1. Check in-memory cache
                            let cache = session_cache.read().await;
                            if cache.contains(token) {
                                // 2. Resolve user from DB
                                if let Ok(user) = crate::tables::user_db::get_user_by_token(&pool, token).await {
                                    chat_msg.user_id = user.id;
                                    chat_msg.username = user.username;
                                    is_valid = true;
                                }
                            }
                        }

                        if is_valid {
                            if crate::tables::user_db::save_message(&pool, chat_msg.user_id, &chat_msg.content).await.is_ok() 
                            {
                                if let Ok(broadcast_text) = serde_json::to_string(&chat_msg) {
                                    tx.send(broadcast_text).expect("Failed to broadcast message");
                                }
                            } else {
                                println!("Failed to save message from session");
                            }
                        } else {
                            println!("Invalid session token: {:?}", chat_msg.session_id);
                        }
                    } else {
                        println!("Failed to parse message: {}", text);
                    }
                }
            },
            Err(_e) => break,
        }
    }
}