use crate::error::HikariError;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub discord_token: String,
    pub database_url: String,
    pub redis_url: String,
    pub prefix: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, HikariError> {
        dotenvy::dotenv().ok();
        let discord_token =
            env::var("TOKEN").map_err(|_| HikariError::Config("TOKEN not set".to_string()))?;
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| HikariError::Config("DATABASE_URL not set".to_string()))?;
        let redis_url = env::var("REDIS_URL")
            .map_err(|_| HikariError::Config("REDIS_URL not set".to_string()))?;

        let prefix =
            env::var("PREFIX").map_err(|_| HikariError::Config("PREFIX not set".to_string()))?;

        Ok(Self {
            discord_token,
            database_url,
            redis_url,
            prefix,
        })
    }
}
