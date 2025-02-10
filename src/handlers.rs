use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use futures_util::TryStreamExt;
use mongodb::{
    bson::{self, doc},
    options::ClientOptions,
    Client, Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio;
use tracing::error;

use crate::models::{AppState, Exercise, ExerciseLog, GymSession, Set, User};

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<User>,
) -> Result<Json<User>, StatusCode> {
    let users_collection = state.db.collection::<User>("users");

    let result = users_collection
        .insert_one(payload.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("{:?}", result);
    println!("{:?}", payload);
    Ok(Json(payload))
}

pub async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, StatusCode> {
    let users_collection = state.db.collection::<User>("users");

    // Create an empty filter to retrieve all documents
    let filter = doc! {};

    // Execute the find operation
    let mut cursor = users_collection
        .find(filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);

    // Iterate over the cursor to access each user document
    let mut users = Vec::new();
    while let Ok(user) = cursor
        .as_mut()
        .unwrap()
        .try_next()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    {
        match user {
            None => break,
            Some(user) => users.push(user),
        }
    }

    Ok(Json(users))
}

pub async fn get_last_3_for_user(
    State(state): State<AppState>,
    Path((name, exercise_name)): Path<(String, String)>,
) -> Result<Json<Vec<Set>>, StatusCode> {
    println!(
        "Getting last 3 sets for user: {} and exercise: {}",
        name, exercise_name
    );
    let users_collection: Collection<bson::Document> = state.db.collection("users");

    let pipeline = vec![
        doc! { "$match": { "name": &name } }, // Step 1: Match user
        doc! { "$unwind": "$gym_sessions" },  // Step 2: Unwind gym_sessions
        doc! { "$unwind": "$gym_sessions.exercises" }, // Step 3: Unwind exercises
        doc! { "$match": { "gym_sessions.exercises.exercise.name": &exercise_name } }, // Step 4: Filter exercise
        doc! { "$unwind": "$gym_sessions.exercises.sets" }, // Step 5: Unwind sets
        doc! { "$sort": { "gym_sessions.date": -1 } },      // Step 6: Sort by latest date
        doc! { "$limit": 3 },                               // Step 7: Get the last 3 sets
        doc! {
            "$project": {
                "_id": 0,
                "weight": "$gym_sessions.exercises.sets.weight",
                "reps": { "$toLong": "$gym_sessions.exercises.sets.reps" }, // Ensure reps is an integer
                "struggle_score": "$gym_sessions.exercises.sets.struggle_score",
            }
        },
    ];

    let mut cursor = users_collection.aggregate(pipeline).await.map_err(|e| {
        println!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut results = Vec::new();
    while let Some(doc) = cursor.try_next().await.map_err(|e| {
        println!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })? {
        println!("{:?}", doc);
        let exercise_log: Set = bson::from_document(doc).map_err(|e| {
            println!("{:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        results.push(exercise_log);
    }

    Ok(Json(results))
}

pub async fn get_user_sessions(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<GymSession>>, StatusCode> {
    let users_collection = state.db.collection::<User>("users");

    let user = users_collection
        .find_one(doc! { "name": name })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("{:?}", user);
    if let Some(user) = user {
        Ok(Json(user.gym_sessions))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn add_user_session(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(new_session): Json<GymSession>,
) -> Result<StatusCode, StatusCode> {
    println!("Adding session for user: {}", name);
    let users_collection = state.db.collection::<User>("users");

    let update_result = users_collection
        .update_one(
            doc! { "name": name },
            doc! { "$push": { "gym_sessions": bson::to_bson(&new_session).unwrap() } },
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("{:?}", update_result);
    if update_result.matched_count == 1 {
        Ok(StatusCode::OK)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn add_excercise(
    State(state): State<AppState>,
    Json(new_excercise): Json<Exercise>,
) -> Result<StatusCode, StatusCode> {
    let exercises_collection = state.db.collection::<Exercise>("all_exercises");

    let result = exercises_collection
        .insert_one(new_excercise.clone())
        .await
        .map_err(|e| {
            error!("Failed to insert exercise: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    println!("{:?}", result);
    println!("{:?}", new_excercise);
    Ok(StatusCode::OK)
}

pub async fn get_exercises(State(state): State<AppState>) -> Result<Json<Vec<Exercise>>, StatusCode> {
    let exercises_collection = state.db.collection::<Exercise>("all_exercises");
    let filter = doc! {};

    let mut cursor = exercises_collection
        .find(filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut exercises = Vec::new();
    while let Some(exercise) = cursor
        .try_next()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        exercises.push(exercise);
    }

    Ok(Json(exercises))
}
