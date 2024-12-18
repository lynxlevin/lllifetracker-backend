use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240722_000001_create_users_table::User;

const INDEX_NAME: &str = "mission_memos_user_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MissionMemo::Table)
                    .if_not_exists()
                    .col(uuid(MissionMemo::Id).primary_key())
                    .col(uuid(MissionMemo::UserId))
                    .col(string_len(MissionMemo::Title, 64))
                    .col(text(MissionMemo::Text))
                    .col(date(MissionMemo::Date))
                    .col(boolean(MissionMemo::Archived).default(false))
                    .col(
                        timestamp_with_time_zone_null(MissionMemo::AccomplishedAt),
                    )
                    .col(
                        timestamp_with_time_zone(MissionMemo::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(MissionMemo::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mission-memos-user_id")
                            .from(MissionMemo::Table, MissionMemo::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(INDEX_NAME)
                    .table(MissionMemo::Table)
                    .col(MissionMemo::UserId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MissionMemo::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum MissionMemo {
    Table,
    Id,
    UserId,
    Title,
    Text,
    Date,
    Archived,
    AccomplishedAt,
    CreatedAt,
    UpdatedAt,
}
