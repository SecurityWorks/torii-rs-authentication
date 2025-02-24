use dashmap::DashMap;
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use torii::Torii;
use torii_auth_oauth::providers::Provider;
use torii_storage_sqlite::SqliteStorage;

mod routes;
mod templates;

/// Application state shared between route handlers
/// Contains references to:
/// - plugin_manager: Coordinates authentication plugins
#[derive(Clone)]
pub(crate) struct AppState {
    torii: Arc<Torii<SqliteStorage, SqliteStorage>>,
    todos: Arc<DashMap<String, Todo>>,
}

#[derive(Debug, Clone)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub completed_at: Option<String>,
    pub user_id: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let pool = Pool::<Sqlite>::connect("sqlite://todos.db?mode=rwc")
        .await
        .expect("Failed to connect to database");

    let torii = Torii::new(
        Arc::new(SqliteStorage::new(pool.clone())),
        Arc::new(SqliteStorage::new(pool.clone())),
    )
    .with_email_password_plugin()
    .with_oauth_provider(Provider::google(
        std::env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID is not set"),
        std::env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET is not set"),
        std::env::var("GOOGLE_REDIRECT_URI").expect("GOOGLE_REDIRECT_URI is not set"),
    ))
    .with_passkey_plugin("rp_id", "rp_origin");

    let app_state = AppState {
        torii: Arc::new(torii),
        todos: Arc::new(DashMap::new()),
    };

    let app = routes::create_router(app_state);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
            .await
            .expect("Failed to bind to port");
        println!(
            "Listening on {}",
            listener.local_addr().expect("Failed to get local address")
        );
        axum::serve(listener, app).await.expect("Server error");
    });

    println!("Please open the following URL in your browser: http://localhost:4000/");
    println!("Press Enter or Ctrl+C to exit...");
    let _ = std::io::stdin().read_line(&mut String::new());
}
