use warp::Filter;
#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub fn login_route(
    pool: sqlx::MySqlPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("login"))
        .and(warp::post()) // Intercept only POST requests
        .and(warp::body::json()) // Automatically parse JSON into LoginRequest
        .and(warp::any().map(move || pool.clone())) // Inject the database pool
        .and_then(handle_login) // Pass the data to your logic function
}

pub async fn handle_login(
    auth: LoginRequest,
    pool: sqlx::MySqlPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_result = crate::tables::user_db::find_user_by_username(&pool, &auth.username).await;

    match user_result {
        Ok(user_result)
            if crate::tables::user_db::verify_password(
                &auth.password,
                &user_result.password_hash,
            )
            .is_ok() =>
        {
            // Success: Return JSON with 200 OK
            Ok(warp::reply::with_status(
                warp::reply::json(&"Login successful!"),
                warp::http::StatusCode::OK,
            ))
        }
        _ => {
            // Failure: Return JSON with 401 Unauthorized
            Ok(warp::reply::with_status(
                warp::reply::json(&"Invalid username or password"),
                warp::http::StatusCode::UNAUTHORIZED,
            ))
        }
    }
}
