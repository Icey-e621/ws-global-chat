use warp::Filter;
use crate::tables::user_db::{create_user, create_session, get_user_by_session_token, delete_session};

#[derive(serde::Deserialize)]
pub struct LimitMessages {
    pub limit: i32,
}
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

    if let Ok(user) = user_result {
        if crate::tables::user_db::verify_password(&auth.password, &user.password_hash).is_ok() {
            let user_id = user.id;
            match create_session(&pool, user_id).await {
                Ok(token) => {
                    let cookie = format!("session_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800", token);
                    return Ok(warp::reply::with_header(
                        warp::reply::with_status(
                            warp::reply::json(&AuthResponse {
                                user_id,
                                message: "Login successful!".to_string(),
                            }),
                            warp::http::StatusCode::OK,
                        ),
                        "Set-Cookie",
                        cookie,
                    ));
                }
                Err(_) => {
                    return Ok(warp::reply::with_header(
                        warp::reply::with_status(
                            warp::reply::json(&"Failed to create session"),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        ),
                        "Set-Cookie",
                        "",
                    ));
                }
            }
        }
    }

    // Default failure case
    Ok(warp::reply::with_header(
        warp::reply::with_status(
            warp::reply::json(&"Invalid username or password"),
            warp::http::StatusCode::UNAUTHORIZED,
        ),
        "Set-Cookie",
        "",
    ))
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
            Ok(warp::reply::with_header(
                warp::reply::with_status(
                    warp::reply::json(&"User already exists"),
                    warp::http::StatusCode::CONFLICT,
                ),
                "Set-Cookie",
                "",
            ))
        }
        _ => {
            match create_user(&pool, &auth.username, &auth.password).await {
                Ok(_) => {
                    // Fetch the user to get the ID
                    if let Ok(user) = crate::tables::user_db::find_user_by_username(&pool, &auth.username).await {
                        let user_id = user.id;
                        match create_session(&pool, user_id).await {
                            Ok(token) => {
                                let cookie = format!("session_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800", token);
                                Ok(warp::reply::with_header(
                                    warp::reply::with_status(
                                        warp::reply::json(&AuthResponse {
                                            user_id,
                                            message: "Registered successfully".to_string(),
                                        }),
                                        warp::http::StatusCode::OK,
                                    ),
                                    "Set-Cookie",
                                    cookie,
                                ))
                            }
                            Err(_) => Ok(warp::reply::with_header(
                                warp::reply::with_status(
                                    warp::reply::json(&"Registered but failed to create session"),
                                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                                ),
                                "Set-Cookie",
                                "",
                            )),
                        }
                    } else {
                        Ok(warp::reply::with_header(
                            warp::reply::with_status(
                                warp::reply::json(&"Registered but failed to fetch ID"),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            ),
                            "Set-Cookie",
                            "",
                        ))
                    }
                }
                Err(_) => Ok(warp::reply::with_header(
                    warp::reply::with_status(
                        warp::reply::json(&"Database is not online, please try again later"),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ),
                    "Set-Cookie",
                    "",
                )),
            }
        }
    }
}

pub fn get_chat_history(
    pool: sqlx::MySqlPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("get_chat_history"))
        .and(warp::get()) // Intercept only POST requests
        .and(warp::query::query())
        .and(warp::any().map(move || pool.clone())) // Inject the database pool
        .and_then(handle_chat_history) // Pass the data to your logic function
}

pub async fn handle_chat_history(
    limit: LimitMessages,
    pool: sqlx::MySqlPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    let chat_history = crate::tables::user_db::get_chat_history(&pool, limit.limit).await;

    match chat_history {
        Ok(messages_vector) => Ok(warp::reply::with_header(
            warp::reply::with_status(
                warp::reply::json(&messages_vector),
                warp::http::StatusCode::OK,
            ),
            "Set-Cookie",
            "",
        )),
        _ => Ok(warp::reply::with_header(
            warp::reply::with_status(
                warp::reply::json(&"Database is not online, please try again later"),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
            "Set-Cookie",
            "",
        )),
    }
}

pub fn get_me_route(
    pool: sqlx::MySqlPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("me"))
        .and(warp::get())
        .and(warp::header::optional::<String>("cookie"))
        .and(warp::any().map(move || pool.clone()))
        .and_then(handle_get_me)
}

pub async fn handle_get_me(
    cookie_header: Option<String>,
    pool: sqlx::MySqlPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(cookie_str) = cookie_header {
        if let Some(token) = extract_session_token(&cookie_str) {
            if let Ok(user) = get_user_by_session_token(&pool, &token).await {
                return Ok(warp::reply::with_header(
                    warp::reply::with_status(
                        warp::reply::json(&user),
                        warp::http::StatusCode::OK,
                    ),
                    "Set-Cookie",
                    "",
                ));
            }
        }
    }

    Ok(warp::reply::with_header(
        warp::reply::with_status(
            warp::reply::json(&"Not logged in"),
            warp::http::StatusCode::UNAUTHORIZED,
        ),
        "Set-Cookie",
        "",
    ))
}

pub fn logout_route(
    pool: sqlx::MySqlPool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("logout"))
        .and(warp::post())
        .and(warp::header::optional::<String>("cookie"))
        .and(warp::any().map(move || pool.clone()))
        .and_then(handle_logout)
}

pub async fn handle_logout(
    cookie_header: Option<String>,
    pool: sqlx::MySqlPool,
) -> Result<impl warp::Reply, warp::Rejection> {
    if let Some(cookie_str) = cookie_header {
        if let Some(token) = extract_session_token(&cookie_str) {
            let _ = delete_session(&pool, &token).await;
        }
    }

    let cookie = "session_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    Ok(warp::reply::with_header(
        warp::reply::with_status(
            warp::reply::json(&"Logged out"),
            warp::http::StatusCode::OK,
        ),
        "Set-Cookie",
        cookie,
    ))
}

fn extract_session_token(cookie_str: &str) -> Option<String> {
    for cookie in cookie_str.split(';') {
        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
        if parts.len() == 2 && parts[0] == "session_token" {
            return Some(parts[1].to_string());
        }
    }
    None
}
