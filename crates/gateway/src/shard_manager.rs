use hikari_utils::error::HikariError;
use twilight_gateway::{ConfigBuilder, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;

pub struct ShardManager {
    pub shards: Vec<Shard>,
}

impl ShardManager {
    pub async fn new(token: String, http: &HttpClient) -> Result<Self, HikariError> {
        let intents = Intents::GUILDS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT
            | Intents::GUILD_MODERATION
            | Intents::GUILD_MEMBERS;

        let base_config = ConfigBuilder::new(token, intents).identify_properties(
            twilight_model::gateway::payload::outgoing::identify::IdentifyProperties::new(
                "Discord VR",
                "twilight.rs",
                std::env::consts::OS,
            ),
        );

        let shards_iter = twilight_gateway::create_recommended(
            http,
            base_config.build(),
            |shard_id: ShardId, builder: ConfigBuilder| {
                let activity = twilight_model::gateway::presence::Activity {
                    application_id: None,
                    assets: None,
                    buttons: Vec::new(),
                    created_at: None,
                    details: None,
                    emoji: None,
                    flags: None,
                    id: None,
                    instance: None,
                    kind: twilight_model::gateway::presence::ActivityType::Custom,
                    name: "Custom Status".into(),
                    party: None,
                    secrets: None,
                    state: Some(format!("🔗 /help · hikari · ✦ cluster {}", shard_id.number())),
                    timestamps: None,
                    url: None,
                };

                let presence = twilight_model::gateway::payload::outgoing::update_presence::UpdatePresencePayload::new(
                    vec![activity],
                    false,
                    None,
                    twilight_model::gateway::presence::Status::Online,
                ).ok();

                if let Some(p) = presence {
                    builder.presence(p).build()
                } else {
                    builder.build()
                }
            },
        )
        .await
        .map_err(|e| HikariError::Internal(format!("Failed to create recommended shards: {}", e)))?;

        let shards: Vec<Shard> = shards_iter.collect();

        Ok(Self { shards })
    }
}
