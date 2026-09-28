#[derive(thiserror::Error, Debug)]
pub enum HikariError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cache error: {0}")]
    Cache(#[from] redis::RedisError),

    #[error("Discord HTTP error: {0}")]
    DiscordHttp(Box<twilight_http::Error>),

    #[error("Discord model error: {0}")]
    DiscordModel(Box<twilight_http::response::DeserializeBodyError>),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Module error: {0}")]
    Module(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<twilight_http::Error> for HikariError {
    fn from(err: twilight_http::Error) -> Self {
        Self::DiscordHttp(Box::new(err))
    }
}

impl From<twilight_http::response::DeserializeBodyError> for HikariError {
    fn from(err: twilight_http::response::DeserializeBodyError) -> Self {
        Self::DiscordModel(Box::new(err))
    }
}
