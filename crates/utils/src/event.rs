use twilight_model::gateway::event::Event;

#[derive(Debug, Clone)]
pub enum HikariEvent {
    Discord(Box<Event>),
    AntinukeThresholdExceeded {
        guild_id: twilight_model::id::Id<twilight_model::id::marker::GuildMarker>,
    },
    Synthetic(String),
}

impl From<Event> for HikariEvent {
    fn from(event: Event) -> Self {
        Self::Discord(Box::new(event))
    }
}
