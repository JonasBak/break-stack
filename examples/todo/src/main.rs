mod components;
mod controllers;
mod models;
mod routes;

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use break_stack::auth::UserId;
use break_stack::errors::*;
use break_stack::hot_reload;
use break_stack::models::DBConn;
use sqlx::sqlite::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    db_pool: SqlitePool,
}

impl AppState {
    pub async fn new(db_url: &str) -> AppState {
        let db_pool = SqlitePool::connect(db_url).await.unwrap();
        AppState { db_pool }
    }
    pub async fn conn(&self) -> sqlx::Result<DBConn> {
        self.db_pool.acquire().await
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish()
    }
}

#[async_trait]
impl FromRequestParts<AppState> for DBConn {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(state.conn().await?)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for UserId {
    type Rejection = AppError;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Err(AppError::Auth(AuthError::Unauthenticated))
    }
}

#[tokio::main]
async fn main() {
    let app_state = AppState::new(&std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        println!("WARNING: DATABASE_URL not set, using in memory database");
        "sqlite::memory:".to_string()
    }))
    .await;

    {
        let mut conn = app_state.conn().await.unwrap();
        sqlx::migrate!().run(&mut conn).await.unwrap();
    }

    let app = routes::router()
        .nest("/reload", hot_reload::reload_router())
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
