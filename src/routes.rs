use tokio::sync::broadcast;
use warp::Filter;

pub fn ws_route(
    pool: sqlx::MySqlPool,
    tx: broadcast::Sender<String>
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("ws")
        .and(warp::ws())
        .and(with_db(pool))
        .and(with_broadcast(tx.clone()))
        .map(| ws: warp::ws::Ws, pool: sqlx::MySqlPool, tx: broadcast::Sender<String>| {
            let pool_for_task = pool.clone(); 
            ws.on_upgrade(move |websocket| {
                // Inside your handler, you'll call tx.subscribe() to get a Receiver
                crate::ws_handler::handle_connection(pool_for_task,websocket, tx)
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
