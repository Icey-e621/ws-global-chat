use warp::Filter;

use crate::tables::user_db::create_user;
#[derive(serde::Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize)]
pub struct AuthResponse {
    pub user_id: i32,
    pub message: String,
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
            // Success: Return JSON with 200 OK and user_id
            Ok(warp::reply::with_status(
                warp::reply::json(&AuthResponse {
                    user_id: user_result.id,
                    message: "Login successful!".to_string(),
                }),
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

pub fn register_route(
    pool: sqlx::MySqlPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("register"))
        .and(warp::post()) // Intercept only POST requests
        .and(warp::body::json()) // Automatically parse JSON into LoginRequest
        .and(warp::any().map(move || pool.clone())) // Inject the database pool
        .and_then(handle_register) // Pass the data to your logic function
}

pub async fn handle_register(
    auth: LoginRequest,
    pool: sqlx::MySqlPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let user_result = crate::tables::user_db::find_user_by_username(&pool, &auth.username).await;

    match user_result {
        Ok(_) => {
            // Success: Return JSON with 409 conflicting data
            Ok(warp::reply::with_status(
                warp::reply::json(&"User already exists"),
                warp::http::StatusCode::CONFLICT,
            ))
        }
        _ => {
            match create_user(&pool, &auth.username, &auth.password).await {
                Ok(_) => {
                    // Fetch the user to get the ID
                    if let Ok(user) = crate::tables::user_db::find_user_by_username(&pool, &auth.username).await {
                        Ok(warp::reply::with_status(
                            warp::reply::json(&AuthResponse {
                                user_id: user.id,
                                message: "Registered successfully".to_string(),
                            }),
                            warp::http::StatusCode::OK,
                        ))
                    } else {
                        Ok(warp::reply::with_status(
                            warp::reply::json(&"Registered but failed to fetch ID"),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ))
                    }
                }
                Err(_) => {
                    Ok(warp::reply::with_status(
                        warp::reply::json(&"Database is not online, please try again later"),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
    }
}
