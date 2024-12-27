use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240722_000001_create_users_table::User;
use crate::m20240927_000003_create_actions_table::Action;

const STARTED_AT_INDEX_NAME: &str = "action_tracks_user_id_started_at_index";
const ACTION_INDEX_NAME: &str = "action_tracks_user_id_action_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ActionTrack::Table)
                    .if_not_exists()
                    .col(uuid(ActionTrack::Id).primary_key())
                    .col(uuid(ActionTrack::UserId))
                    .col(uuid_null(ActionTrack::ActionId))
                    .col(
                        timestamp_with_time_zone(ActionTrack::StartedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(ActionTrack::EndedAt))
                    // NOTE: Using interval column results in "not implemented" error on `sea-orm-cli generate entity`.
                    .col(big_integer_null(ActionTrack::Duration))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-action_tracks-user_id")
                            .from(ActionTrack::Table, ActionTrack::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-action_tracks-action_id")
                            .from(ActionTrack::Table, ActionTrack::ActionId)
                            .to(Action::Table, Action::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(STARTED_AT_INDEX_NAME)
                    .table(ActionTrack::Table)
                    .col(ActionTrack::UserId)
                    .col(ActionTrack::StartedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(ACTION_INDEX_NAME)
                    .table(ActionTrack::Table)
                    .col(ActionTrack::UserId)
                    .col(ActionTrack::ActionId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(ACTION_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(STARTED_AT_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ActionTrack::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ActionTrack {
    Table,
    Id,
    UserId,
    ActionId,
    StartedAt,
    EndedAt,
    Duration,
}
