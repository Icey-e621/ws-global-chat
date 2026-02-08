use tokio::sync::broadcast;
use warp::Filter;

use crate::{api::{login_route, register_route}, routes::ws_route};
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

    let total_route = ws_route.or(login_route).or(register_route);

    //Serve
    warp::serve(total_route).run(([0, 0, 0, 0], 8000)).await;
    Ok(())
}
