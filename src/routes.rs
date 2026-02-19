use tokio::sync::{broadcast, RwLock};
use warp::Filter;
use std::sync::Arc;
use std::collections::HashSet;

pub fn ws_route(
    pool: sqlx::MySqlPool,
    tx: broadcast::Sender<String>,
    session_cache: Arc<RwLock<HashSet<String>>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .and(with_db(pool))
        .and(with_broadcast(tx.clone()))
        .and(with_session_cache(session_cache))
        .map(| ws: warp::ws::Ws, pool: sqlx::MySqlPool, tx: broadcast::Sender<String>, cache: Arc<RwLock<HashSet<String>>>| {
            let pool_for_task = pool.clone(); 
            ws.on_upgrade(move |websocket| {
                crate::ws_handler::handle_connection(pool_for_task, websocket, tx, cache)
            })
        })
}

// Create a filter that yields a clone of the pool
fn with_db(
    pool: sqlx::MySqlPool,
) -> impl Filter<Extract = (sqlx::MySqlPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}
fn with_broadcast(
    tx: broadcast::Sender<String>,
) -> impl Filter<Extract = (broadcast::Sender<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || tx.clone())
}

fn with_session_cache(
    cache: Arc<RwLock<HashSet<String>>>,
) -> impl Filter<Extract = (Arc<RwLock<HashSet<String>>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cache.clone())
}
