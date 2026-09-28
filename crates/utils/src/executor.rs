use std::sync::Arc;
use tokio::sync::mpsc;
use twilight_http::Client as HttpClient;
use twilight_model::id::{Id, marker::*};

pub struct BanExecutor {
    tx: mpsc::UnboundedSender<BanRequest>,
}

struct BanRequest {
    guild_id: u64,
    user_id: u64,
}

impl BanExecutor {
    pub fn new(http: Arc<HttpClient>) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<BanRequest>();

        tokio::spawn(async move {
            while let Some(req) = rx.recv().await {
                let http_clone = http.clone();
                tokio::spawn(async move {
                    let _ = http_clone
                        .create_ban(
                            Id::<GuildMarker>::new(req.guild_id),
                            Id::<UserMarker>::new(req.user_id),
                        )
                        .delete_message_seconds(0)
                        .await;
                });
            }
        });

        Self { tx }
    }

    #[inline(always)]
    pub fn execute(&self, guild_id: u64, user_id: u64) {
        let _ = self.tx.send(BanRequest { guild_id, user_id });
    }
}

impl Clone for BanExecutor {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub struct ConnectionWarmer;

impl ConnectionWarmer {
    pub async fn warm_connections(http: Arc<HttpClient>, warmup_count: u8) {
        tokio::spawn(async move {
            for _ in 0..warmup_count {
                let _ = http.current_user().await;
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
    }
}
