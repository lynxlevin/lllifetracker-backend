use sea_orm_migration::prelude::*;


const UNIQUE_STARTED_AT_INDEX_NAME: &str = "action_tracks_user_id_action_id_started_at_unique_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(UNIQUE_STARTED_AT_INDEX_NAME)
                    .table(ActionTrack::Table)
                    .col(ActionTrack::UserId)
                    .col(ActionTrack::ActionId)
                    .col(ActionTrack::StartedAt)
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(UNIQUE_STARTED_AT_INDEX_NAME).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ActionTrack {
    Table,
    UserId,
    ActionId,
    StartedAt,
}
