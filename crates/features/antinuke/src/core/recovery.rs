use super::snapshot::{ChannelSnap, RecoveryKind, RecoveryTask, RoleSnap};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use twilight_http::Client as HttpClient;
use twilight_model::id::Id;

pub struct RecoverySystem {
    tx: mpsc::UnboundedSender<RecoveryTask>,
}

impl RecoverySystem {
    pub fn new(http: Arc<HttpClient>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let system = Self { tx };

        system.spawn_worker(rx, http);
        system
    }

    fn spawn_worker(&self, mut rx: mpsc::UnboundedReceiver<RecoveryTask>, http: Arc<HttpClient>) {
        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(3));

            while let Some(task) = rx.recv().await {
                let permit = semaphore.clone().acquire_owned().await.ok();
                if permit.is_none() {
                    continue;
                }

                let http_clone = http.clone();
                tokio::spawn(async move {
                    let _permit = permit;
                    Self::execute_recovery(http_clone, task).await;
                    tokio::time::sleep(Duration::from_millis(50)).await;
                });
            }
        });
    }

    pub async fn enqueue(&self, task: RecoveryTask) {
        let _ = self.tx.send(task);
    }

    async fn execute_recovery(http: Arc<HttpClient>, task: RecoveryTask) {
        let result = match &task.kind {
            RecoveryKind::Channel(snap) => Self::recover_channel(&http, task.guild_id, snap).await,
            RecoveryKind::Role(snap) => Self::recover_role(&http, task.guild_id, snap).await,
            RecoveryKind::MemberRoles { user_id, role_ids } => {
                Self::recover_member_roles(&http, task.guild_id, *user_id, role_ids).await
            }
        };

        if let Err(e) = result
            && Self::is_rate_limit(&e)
        {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let _ = match &task.kind {
                RecoveryKind::Channel(snap) => {
                    Self::recover_channel(&http, task.guild_id, snap).await
                }
                RecoveryKind::Role(snap) => Self::recover_role(&http, task.guild_id, snap).await,
                RecoveryKind::MemberRoles { user_id, role_ids } => {
                    Self::recover_member_roles(&http, task.guild_id, *user_id, role_ids).await
                }
            };
        }
    }

    async fn recover_channel(
        http: &HttpClient,
        guild_id: u64,
        snap: &ChannelSnap,
    ) -> Result<(), twilight_http::Error> {
        let mut req = http
            .create_guild_channel(Id::new(guild_id), &snap.name)
            .kind(twilight_model::channel::ChannelType::from(snap.kind))
            .position(snap.position as u64)
            .nsfw(snap.nsfw)
            .rate_limit_per_user(snap.rate_limit_per_user);

        if let Some(t) = &snap.topic {
            req = req.topic(t);
        }
        if let Some(pid) = snap.parent_id {
            req = req.parent_id(Id::new(pid));
        }

        let response = req.await?;
        if let Ok(new_channel) = response.model().await {
            for ow in &snap.overwrites {
                use twilight_model::http::permission_overwrite::{
                    PermissionOverwrite, PermissionOverwriteType,
                };

                let kind = match ow.kind {
                    0 => PermissionOverwriteType::Role,
                    1 => PermissionOverwriteType::Member,
                    _ => continue,
                };

                let allow = twilight_model::guild::Permissions::from_bits_truncate(ow.allow);
                let deny = twilight_model::guild::Permissions::from_bits_truncate(ow.deny);

                let perm_ow = PermissionOverwrite {
                    allow: Some(allow),
                    deny: Some(deny),
                    id: Id::new(ow.id),
                    kind,
                };

                let _ = http
                    .update_channel_permission(new_channel.id, &perm_ow)
                    .await;

                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }

        Ok(())
    }

    async fn recover_role(
        http: &HttpClient,
        guild_id: u64,
        snap: &RoleSnap,
    ) -> Result<(), twilight_http::Error> {
        http.create_role(Id::new(guild_id))
            .name(&snap.name)
            .color(snap.color)
            .permissions(twilight_model::guild::Permissions::from_bits_truncate(
                snap.permissions,
            ))
            .hoist(snap.hoist)
            .mentionable(snap.mentionable)
            .await?;
        Ok(())
    }

    async fn recover_member_roles(
        http: &HttpClient,
        guild_id: u64,
        user_id: u64,
        role_ids: &[u64],
    ) -> Result<(), twilight_http::Error> {
        let roles: Vec<Id<twilight_model::id::marker::RoleMarker>> =
            role_ids.iter().map(|&id| Id::new(id)).collect();

        http.update_guild_member(Id::new(guild_id), Id::new(user_id))
            .roles(&roles)
            .await?;
        Ok(())
    }

    #[inline]
    fn is_rate_limit(error: &twilight_http::Error) -> bool {
        matches!(
            error.kind(),
            twilight_http::error::ErrorType::Response {
                status: twilight_http::response::StatusCode::TOO_MANY_REQUESTS,
                ..
            }
        )
    }
}
