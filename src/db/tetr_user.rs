use sqlx::{sqlite::SqlitePoolOptions, SqlitePool, query, query_as};
use sqlx::types::chrono::{DateTime, Utc};

use serenity::prelude::TypeMapKey;

use tetr_ch::model::{
    user::{
        UserRecordsResponse, 
        UserResponse,
        User}, 
    league::Rank,
    stream::StreamResponse};

#[derive(Debug, Clone)]
pub struct TetrUser {
    pub user_id: i64,
    pub id: String,
    pub last_update: DateTime<Utc>,
    pub tr: f64,
    pub rank: String,
    pub apm: Option<f64>,
    pub pps: Option<f64>,
    pub vs: Option<f64>,
    pub sprint: Option<f64>,
    pub blitz: Option<f64>,
}

impl TetrUser {
    pub fn new(user_id: i64, id: String, last_update: DateTime<Utc>, tr: f64, rank: String, apm: Option<f64>, pps: Option<f64>, vs: Option<f64>, sprint: Option<f64>, blitz: Option<f64>) -> Self {
        Self {
            user_id,
            id,
            last_update,
            tr,
            rank,
            apm,
            pps,
            vs,
            sprint,
            blitz
        }
    }

    pub fn get_stat(&self, stat: &str) -> Option<f64> {
        match stat {
            "tr" => Some(self.tr.clone()),
            "apm" => self.apm.clone(),
            "pps" => self.pps.clone(),
            "vs" => self.vs.clone(),
            "app" if matches!(self.apm, Some(_)) && matches!(self.pps, Some(_)) => Some(self.apm.unwrap() / self.pps.unwrap() / 60.0),
            "40l" | "sprint" => self.sprint.clone(),
            "blitz" => self.blitz.clone(),
            _ => None
        }
    }
}

pub struct TetrSavedUsers {
    pool: SqlitePool,
}

impl TypeMapKey for TetrSavedUsers {
    type Value = TetrSavedUsers;
}

impl TetrSavedUsers {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TetrSavedUsers {
    // pub async fn by_user_id(&self, user_id: i64) -> Result<Option<TetrUser>, sqlx::Error> {
    //     let u = query_as!(
    //         TetrUser,
    //         "SELECT 
    //             user_id as 'user_id: i64',
    //             id as 'id: String',
    //             last_update as 'last_update: DateTime<Utc>',
    //             tr, rank, apm, pps, vs, sprint, blitz
    //         FROM tetr_users WHERE user_id = ?", 
    //         user_id)
    //         .fetch_optional(&self.pool)
    //         .await?;

    //     Ok(u)
    // }

    // pub async fn by_tetr_id(&self, tetr_id: &str) -> Result<Option<TetrUser>, sqlx::Error> {
    //     let u = query_as!(
    //         TetrUser, 
    //         "SELECT 
    //             user_id as 'user_id: i64',
    //             id as 'id: String',
    //             last_update as 'last_update: DateTime<Utc>',
    //             tr, rank, apm, pps, vs, sprint, blitz
    //         FROM tetr_users WHERE id = ?", 
    //         tetr_id)
    //         .fetch_optional(&self.pool)
    //         .await?;

    //     Ok(u)
    // }

    pub async fn all(&self) -> Result<Vec<TetrUser>, sqlx::Error> {
        let u = query_as!(
            TetrUser, 
            "SELECT
                user_id as 'user_id: i64',
                id as 'id: String',
                last_update as 'last_update: DateTime<Utc>',
                tr, rank, apm, pps, vs, sprint, blitz
            FROM tetr_users")
            .fetch_all(&self.pool)
            .await?;

        Ok(u)
    }
}

impl TetrSavedUsers {
    pub async fn save(&self, user: &TetrUser) -> Result<(), sqlx::Error> {
        query!(
            "INSERT 
            INTO tetr_users (user_id, id, last_update, tr, rank, apm, pps, vs, sprint, blitz) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE
            SET 
                id = excluded.id, 
                last_update = excluded.last_update, 
                tr = excluded.tr, 
                rank = excluded.rank,
                apm = excluded.apm, 
                pps = excluded.pps, 
                vs = excluded.vs, 
                sprint = excluded.sprint, 
                blitz = excluded.blitz",
            user.user_id,
            user.id,
            user.last_update,
            user.tr,
            user.rank,
            user.apm,
            user.pps,
            user.vs,
            user.sprint,
            user.blitz
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    
    // pub async fn delete(&self, user_id: i64) -> Result<(), sqlx::Error> {
    //     query!(
    //         "DELETE FROM tetr_users WHERE user_id = ?",
    //         user_id
    //     )
    //     .execute(&self.pool)
    //     .await?;

    //     Ok(())
    // }
}