use tokio::sync::broadcast;
use warp::Filter;

use crate::{login::login_route, routes::ws_route};
//mod ~= namespace import
mod db;
mod login;
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
    let ws_route = ws_route(pool.clone(), tx.clone());

    let total_route = ws_route.or(login_route);

    //Serve
    warp::serve(total_route).run(([0, 0, 0, 0], 8000)).await;
    Ok(())
}
