use sea_orm_migration::{prelude::*, schema::*};

use crate::m20240722_000001_create_users_table::User;
use crate::m20240927_000006_create_tags_table::Tag;

const STARTED_AT_INDEX_NAME: &str = "records_user_id_started_at_index";
const TAG_INDEX_NAME: &str = "records_user_id_tag_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Record::Table)
                    .if_not_exists()
                    .col(uuid(Record::Id).primary_key())
                    .col(uuid(Record::UserId))
                    .col(uuid_null(Record::TagId))
                    .col(string_null(Record::ActionName))
                    .col(
                        timestamp_with_time_zone(Record::StartedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(timestamp_with_time_zone_null(Record::EndedAt))
                    .col(
                        timestamp_with_time_zone(Record::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Record::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-records-user_id")
                            .from(Record::Table, Record::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-records-tag_id")
                            .from(Record::Table, Record::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(STARTED_AT_INDEX_NAME)
                    .table(Record::Table)
                    .col(Record::UserId)
                    .col(Record::StartedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(TAG_INDEX_NAME)
                    .table(Record::Table)
                    .col(Record::UserId)
                    .col(Record::TagId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(TAG_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(STARTED_AT_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Record::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Record {
    Table,
    Id,
    UserId,
    TagId,
    ActionName,
    StartedAt,
    EndedAt,
    CreatedAt,
    UpdatedAt,
}
