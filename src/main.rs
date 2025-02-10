use axum::{
    extract::{Path, State},
    http::{Method, StatusCode},
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
use clap::{Arg, Command, Parser, Subcommand};
use gym_tracker::{
    auth::login,
    handlers::{
        add_excercise, add_user_session, create_user, get_exercises, get_last_3_for_user,
        get_user_sessions, list_users,
    },
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
use tower_http::cors::{AllowHeaders, AllowOrigin, Any, CorsLayer};
use tracing_subscriber::fmt::format::FmtSpan;

// Define User, ExerciseLog, Exercise, etc. (Use the nested structs from earlier)
/// CLI Parser for the Gym Tracker application.
#[derive(Parser)]
#[command(
    name = "Gym Tracker",
    version = "1.0",
    author = "Your Name <your.email@example.com>",
    about = "Tracks gym activities"
)]
struct GymTrackerCli {
    #[command(subcommand)]
    command: GymCommand,
}

/// Enum for subcommands
#[derive(Subcommand)]
enum GymCommand {
    /// Runs the server
    Run {
        #[arg(short, long, default_value = "0.0.0.0:4000")]
        listener: String,
    },
    /// Creates a new user
    MakeUser {
        #[arg(short, long)]
        username: String,
        #[arg(short, long)]
        password: String,
    },
    /// Prints a gym session
    ShowSession,
    Example {
        name: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO) // Set max log level
        .with_span_events(FmtSpan::CLOSE) // Show span lifecycle events
        .with_ansi(true) // Enable color
        .with_line_number(true)
        .init();

    // MongoDB connection
    let mongo_uri = std::env::var("MONGO_URL").expect("MONGO_URL must be set");
    let options = ClientOptions::parse(mongo_uri).await?;
    let client = Client::with_options(options)?;
    let db = client.database("gym_tracker");

    let args = GymTrackerCli::parse();

    match args.command {
        GymCommand::Run { listener } => {
            run(db, listener).await?;
        }
        GymCommand::MakeUser { username, password } => {
            println!("Creating user: {}", username);
            println!("Password: {}", password);
            create_user_console(db, &username, &password).await?;
        }
        GymCommand::ShowSession => {
            show_a_gym_session();
        }
        GymCommand::Example { name } => {
            if name == "Exercise" {
                //let example = Exercise::example();
                //println!("{}", serde_json::to_string_pretty(&example).unwrap());
            } else {
                eprintln!("No example found for: {}", name);
            }
        }
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
    let session_layer = SessionManagerLayer::new(session_store)
        .with_same_site(axum_login::tower_sessions::cookie::SameSite::Strict)
        .with_secure(false)
        .with_http_only(true);

    // Auth service.
    let b = Backend { db: db.clone() };
    let auth_layer = AuthManagerLayerBuilder::new(b, session_layer).build();

    // Create shared state
    let state = AppState { db: Arc::new(db) };

    // CORS layer
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(AllowOrigin::mirror_request()) // Allow all origins, replace with your frontend's URL for security
        .allow_headers(AllowHeaders::mirror_request())
        .allow_credentials(true); // Required for cookies to be sent

    // Define routes
    let app = Router::new()
        .route("/api/users", get(list_users))
        .route(
            "/api/users/{name}/sessions",
            get(get_user_sessions).post(add_user_session),
        )
        .route("/api/last3/{name}/{exercise}", get(get_last_3_for_user))
        .route("/api/exercise", post(add_excercise).get(get_exercises))
        .route_layer(login_required!(Backend, login_url = "/api/login"))
        .route("/api/login", post(login))
        .with_state(state)
        .layer(auth_layer)
        .layer(cors);

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
                muscle_group: vec![MuscleGroup::UpperChest],
                category: ExerciseCategory::Upper,
            },
            sets: vec![
                Set {
                    weight: 50.0,
                    reps: 10,
                    struggle_score: Some(gym_tracker::models::StruggleScore::Easy),
                },
                Set {
                    weight: 50.0,
                    reps: 10,
                    struggle_score: Some(gym_tracker::models::StruggleScore::Easy),
                },
            ],
        }],
        notes: Some("Good session".to_string()),
    };

    let json_session = serde_json::to_string_pretty(&session).unwrap();
    println!("{}", json_session);
}
