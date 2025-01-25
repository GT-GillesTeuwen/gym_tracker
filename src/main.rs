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
use bcrypt::bcrypt;
use chrono::NaiveDate;
use clap::{Arg, Command};
use gym_tracker::{
    auth::login,
    handlers::{add_user_session, create_user, get_user_sessions, list_users},
    models::{
        AppState, Backend, Exercise, ExerciseCategory, ExerciseLog, GymSession, MuscleGroup, Set,
        User,
    },
};
use mongodb::{
    bson::{bson, doc, oid::ObjectId},
    options::ClientOptions,
    Client, Database,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio;
use tower_http::cors::CorsLayer;

// Define User, ExerciseLog, Exercise, etc. (Use the nested structs from earlier)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    // MongoDB connection
    let mongo_uri = std::env::var("MONGO_URL").expect("MONGO_URL must be set");
    let options = ClientOptions::parse(mongo_uri).await?;
    let client = Client::with_options(options)?;
    let db = client.database("gym_tracker");

    let matches = Command::new("Gym Tracker")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Tracks gym activities")
        .subcommand(
            Command::new("run").about("Runs the server").arg(
                Arg::new("listener")
                    .short('l')
                    .long("listener")
                    .value_name("LISTENER")
                    .default_value("0.0.0.0:3000"),
            ),
        )
        .subcommand(
            Command::new("make_user")
                .about("Creates a new user")
                .arg(
                    Arg::new("username")
                        .short('u')
                        .long("username")
                        .value_name("USERNAME")
                        .required(true),
                )
                .arg(
                    Arg::new("password")
                        .short('p')
                        .long("password")
                        .value_name("PASSWORD")
                        .required(true),
                ),
        )
        .subcommand(Command::new("show_session").about("Prints a gym session"))
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("run") {
        let listener = matches.get_one::<String>("listener").unwrap().to_string();
        run(db, listener.to_string()).await?;
    } else if let Some(matches) = matches.subcommand_matches("make_user") {
        let username = matches.get_one::<String>("username").unwrap();
        let password = matches.get_one::<String>("password").unwrap();
        println!("Creating user: {}", username);
        println!("Password: {}", password);
        create_user_console(db, username, password).await?;
    } else if matches.subcommand_matches("show_session").is_some() {
        show_a_gym_session();
    }

    Ok(())
}

pub async fn create_user_console(
    db: Database,
    user_name: &String,
    password: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let users_collection = db.collection::<User>("users");
    let mut rng = rand::thread_rng();
    let salt: [u8; 16] = rng.gen();
    let user = User {
        id: Some(ObjectId::new()),
        name: user_name.clone(),
        pw_hash: bcrypt(16, salt, password.as_bytes()).to_vec(),
        salt: salt.to_vec(),
        gym_sessions: Vec::new(),
    };
    let result = users_collection.insert_one(user.clone()).await?;
    println!("{:?}", result);
    Ok(())
}

pub async fn run(db: Database, listener: String) -> Result<(), Box<dyn std::error::Error>> {
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
        .route("/users", get(list_users))
        .route(
            "/users/{name}/sessions",
            get(get_user_sessions).post(add_user_session),
        )
        .route_layer(login_required!(Backend, login_url = "/login"))
        .route("/login", post(login))
        .with_state(state)
        .layer(auth_layer)
        .layer(CorsLayer::very_permissive());

    // Run server
    println!("Listening on {}", listener);
    let listener = tokio::net::TcpListener::bind(listener).await.unwrap();
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

pub fn show_a_gym_session() {
    let session = GymSession {
        date: NaiveDate::from_ymd(2021, 10, 1),
        exercises: vec![ExerciseLog {
            exercise: Exercise {
                name: "Bench Press".to_string(),
                muscle_group: MuscleGroup::Chest,
                category: ExerciseCategory::Upper,
            },
            sets: vec![
                Set {
                    weight: 50.0,
                    reps: 10,
                    struggle_score: gym_tracker::models::StruggleScore::Easy,
                },
                Set {
                    weight: 50.0,
                    reps: 10,
                    struggle_score: gym_tracker::models::StruggleScore::Easy,
                },
            ],
        }],
        notes: "Good session".to_string(),
    };

    let json_session = serde_json::to_string_pretty(&session).unwrap();
    println!("{}", json_session);
}
