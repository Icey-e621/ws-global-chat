use warp::Filter;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use tokio::sync::broadcast;

//mod ~= namespace import
mod ws_handler;


//declare main thread runs this
#[tokio::main]
async fn main() {
    let tx = Arc::new(Mutex::new(broadcast::channel(100).0));
    let tx_ws = tx.clone();
    let mut valid_users = HashSet::new();
    valid_users.insert(1); // Hardcoded valid ID
    valid_users.insert(2);
    let valid_users = Arc::new(valid_users);

    // on root/wss and websocket connection
    let ws_route = warp::path("ws_standard")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let tx = tx_ws.clone();
            let users = valid_users.clone();
            ws.on_upgrade(move |websocket| ws_handler::handle_connection(websocket, tx, users))
        });

    // filter http only
    let only_http_filter = warp::header::optional::<String>("upgrade")
        .and_then(|upgrade: Option<String>| async move {
            match upgrade {
                // If the upgrade header exists and contains "websocket", reject it
                Some(v) if v.to_lowercase() == "websocket" => {
                    Err(warp::reject::not_found())
                }
                // Otherwise, let the request through
                _ => Ok(()),
            }
        })
        .untuple_one();
    //where to serve from
    let webpage_route = warp::path("ws")
        .and(only_http_filter)
        .and(warp::fs::dir("./html5/"));

    // Start the http server
    let server = warp::serve(webpage_route).run(([127, 0, 0, 1], 8040));

    // Use tokio::spawn to move that future into the background
    tokio::spawn(server);

    warp::serve(ws_route)
        .run(([127, 0, 0, 1], 8080))
        .await;
}