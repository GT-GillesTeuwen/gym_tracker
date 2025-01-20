use std::sync::Arc;

use async_trait::async_trait;
use mongodb::{bson::{doc, oid::ObjectId}, Database};
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;
use axum_login::{AuthUser, AuthnBackend, UserId};
use bcrypt::{verify, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct User {
    pub id: Option<ObjectId>,
    pub name: String,
    pub pw_hash: Vec<u8>,
    pub salt: Vec<u8>,
    pub email: String,
    pub exercise_logs: Vec<ExerciseLog>, // Nest exercise logs directly
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ExerciseLog {
    pub exercise: Exercise, // Nest the exercise directly
    pub sets: Vec<Set>,
    pub date: NaiveDate,
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
    Push,
    Pull,
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
pub struct Backend{
    pub db:Database,
}

#[async_trait]
impl AuthnBackend for Backend{
    type User = User;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        Credentials { user_name,password }: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let users_collection = self.db.collection::<User>("users");

        let user = users_collection
            .find_one(doc! { "name": user_name })
            .await.unwrap().unwrap();

            println!("attempting to authenticate");
        println!("{:?}", user);
        let password = [password.as_bytes(), &user.salt].concat();
        if !verify(&password, &String::from_utf8(user.pw_hash.clone()).unwrap()).unwrap() {
            println!("compared password {:?} to hash {:?}", password, &String::from_utf8(user.pw_hash.clone()).unwrap());
            println!("Password is incorrect");
            return Ok(None);
        }

        Ok(Some(user))
    }

    async fn get_user(
        &self,
        user_id: &UserId<Self>,
    ) -> Result<Option<Self::User>, Self::Error> {
       let users_collection = self.db.collection::<User>("users");

        let user = users_collection
            .find_one(doc! { "_id": user_id })
            .await.unwrap();

        Ok(user)
    }
}
