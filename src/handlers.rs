use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use mongodb::{bson::{self, doc}, options::ClientOptions, Client, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio;
use futures_util::TryStreamExt;


use crate::models::{AppState, ExerciseLog, User};


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

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let users_collection = state.db.collection::<User>("users");

     // Create an empty filter to retrieve all documents
     let filter = doc! {};

     // Execute the find operation
     let mut cursor = users_collection.find(filter).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
 
     // Iterate over the cursor to access each user document
     let mut users=Vec::new();
     while let Ok(user) = cursor.as_mut().unwrap().try_next().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR) {
         users.push(user.unwrap());
     }
 
    Ok(Json(users))
}

pub async fn get_user_logs(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<ExerciseLog>>, StatusCode> {
    let users_collection = state.db.collection::<User>("users");

    let user = users_collection
        .find_one(doc! { "name": name })
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        println!("{:?}", user);
    if let Some(user) = user {
        Ok(Json(user.exercise_logs))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn add_user_log(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(new_log): Json<ExerciseLog>,
) -> Result<StatusCode, StatusCode> {
    let users_collection = state.db.collection::<User>("users");

    let update_result = users_collection
        .update_one(
            doc! { "name": name },
            doc! { "$push": { "exercise_logs": bson::to_bson(&new_log).unwrap() } },
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
