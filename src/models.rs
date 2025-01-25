use std::sync::Arc;

use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use bcrypt::bcrypt;
use bcrypt::{verify, DEFAULT_COST};
use chrono::NaiveDate;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct User {
    pub id: Option<ObjectId>,
    pub name: String,
    pub pw_hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub gym_sessions: Vec<GymSession>, // Nest exercise logs directly
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct GymSession{
    pub date: NaiveDate,
    pub exercises: Vec<ExerciseLog>,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ExerciseLog {
    pub exercise: Exercise, // Nest the exercise directly
    pub sets: Vec<Set>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Set {
    pub weight: f64,
    pub reps: u32,
    pub struggle_score: StruggleScore,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Exercise {
    pub name: String,
    pub muscle_group: MuscleGroup,
    pub category: ExerciseCategory,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum MuscleGroup {
    Chest,
    Back,
    Legs,
    Shoulders,
    Arms,
    Core,
    FullBody,
    Cardio,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum ExerciseCategory {
    Upper,
    Lower,
    Cardio,
    Other,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum StruggleScore {
    Easy,
    Moderate,
    Hard,
    VeryHard,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
}

impl AuthUser for User {
    type Id = ObjectId;

    fn id(&self) -> Self::Id {
        if self.id.is_none() {
            panic!("User id is None");
        }
        self.id.clone().unwrap()
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Credentials {
    pub user_name: String,
    pub password: String,
}

#[derive(Clone)]
pub struct Backend {
    pub db: Database,
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        Credentials {
            user_name,
            password,
        }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let users_collection = self.db.collection::<User>("users");

        let user = users_collection
            .find_one(doc! { "name": user_name })
            .await
            .unwrap()
            .unwrap();

        println!("attempting to authenticate");
        println!("{:?}", user);
        let salt: [u8; 16] = user.salt.clone().try_into().expect("Salt must be 16 bytes");
        let attempt = bcrypt(16, salt, &password.as_bytes()).to_vec();
        println!("attempt: {:?}", attempt);
        println!("pw_hash: {:?}", user.pw_hash);
        if attempt != user.pw_hash {
            println!("Password is incorrect");
            return Ok(None);
        }
        println!("Password is correct");
        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let users_collection = self.db.collection::<User>("users");

        let user = users_collection
            .find_one(doc! { "id": user_id })
            .await
            .unwrap();

        Ok(user)
    }
}
