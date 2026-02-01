use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use futures_util::{SinkExt, StreamExt as _};
use tokio::sync::broadcast;
use warp::filters::ws::{Message, WebSocket};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ChatMessage {
    user_id: usize,
    content: String,
}


pub async fn handle_connection(ws: WebSocket, tx: Arc<Mutex<broadcast::Sender<String>>>, valid_users: Arc<HashSet<usize>>) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut rx = tx.lock().unwrap().subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
    });
    // for every item in the web socket stream
    while let Some(result) = ws_receiver.next().await {
        //do this
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(text) {
                        if valid_users.contains(&chat_msg.user_id) {
                            //lock thread until sender mutex acquired, then sends received message to the broadcast
                            tx.lock().unwrap().send(text.to_string()).expect("Failed to broadcast message");
                        } else {
                            println!("Unauthorized user_id: {}", chat_msg.user_id);
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