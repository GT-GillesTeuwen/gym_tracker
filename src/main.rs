use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use axum_login::{
    login_required,
    tower_sessions::{MemoryStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use gym_tracker::{
    handlers::{add_user_log, create_user, get_user_logs, list_users},
    models::{AppState, Backend},
};
use mongodb::{bson::doc, options::ClientOptions, Client, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio;

// Define User, ExerciseLog, Exercise, etc. (Use the nested structs from earlier)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    // MongoDB connection
    let mongo_uri = std::env::var("MONGO_URL").expect("MONGO_URL must be set");
    let options = ClientOptions::parse(mongo_uri).await?;
    let client = Client::with_options(options)?;
    let db = client.database("fitness");

    // Session layer.
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);

    // Auth service.
    let b = Backend { db: db.clone() };
    let auth_layer = AuthManagerLayerBuilder::new(b, session_layer).build();

    // Create shared state
    let state = AppState { db: Arc::new(db) };

    // Define routes
    let app = Router::new()
        .route("/users", post(create_user).get(list_users))
        .route_layer(login_required!(Backend, login_url = "/login"))
        .route("/users/:name/logs", get(get_user_logs).post(add_user_log))
        .with_state(state)
        .layer(auth_layer);

    // Run server
    println!("Listening on port 3000");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
