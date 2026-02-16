use tokio::sync::broadcast;
use warp::Filter;

use crate::{api::{login_route, register_route, get_chat_history, get_me_route, logout_route}, routes::ws_route};
//mod ~= namespace import
mod db;
mod api;
mod routes;
mod tables;
mod ws_handler;
//declare main thread runs this
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, _rx) = broadcast::channel::<String>(100);
    let pool = db::create_pool().await?;

    //ROUTES
    let login_route = login_route(pool.clone());
    let register_route = register_route(pool.clone());
    let ws_route = ws_route(pool.clone(), tx.clone());
    let me_route = get_me_route(pool.clone());
    let logout_route = logout_route(pool.clone());
    let chat_history_route = get_chat_history(pool.clone()); 

    let total_route = ws_route.or(login_route).or(register_route).or(chat_history_route).or(me_route).or(logout_route);

    // Background task for session cleanup
    let pool_cleanup = pool.clone();
    tokio::spawn(async move {
        loop {
            match crate::tables::user_db::cleanup_expired_sessions(&pool_cleanup).await {
                Ok(count) => {
                    if count > 0 {
                        log::info!("Cleaned up {} expired sessions", count);
                    }
                }
                Err(e) => log::error!("Failed to cleanup sessions: {:?}", e),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    });

    //Serve
    print!("Now serving server, setup successful");
    warp::serve(total_route).run(([0, 0, 0, 0], 8000)).await;
    Ok(())
}
