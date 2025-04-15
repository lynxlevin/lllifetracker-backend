use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, Expr, ForeignKey, ForeignKeyAction, Index, MigrationTrait,
        SchemaManager, Table,
    },
    schema::{
        boolean, date, string_len, text, timestamp_with_time_zone, timestamp_with_time_zone_null,
        uuid,
    },
};

use crate::m20240722_000001_create_users_table::User;

const INDEX_NAME: &str = "memos_user_id_index";
const MISSION_MEMO_INDEX_NAME: &str = "mission_memos_user_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChallengesTags::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(MISSION_MEMO_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Challenge::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(MemosTags::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Memo::Table).to_owned())
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Memo::Table)
                    .if_not_exists()
                    .col(uuid(Memo::Id).primary_key())
                    .col(uuid(Memo::UserId))
                    .col(string_len(Memo::Title, 64))
                    .col(text(Memo::Text))
                    .col(date(Memo::Date))
                    .col(boolean(Memo::Archived).default(false))
                    .col(
                        timestamp_with_time_zone(Memo::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Memo::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-memos-user_id")
                            .from(Memo::Table, Memo::UserId)
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
                    .table(Memo::Table)
                    .col(Memo::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(MemosTags::Table)
                    .if_not_exists()
                    .col(uuid(MemosTags::MemoId))
                    .col(uuid(MemosTags::TagId))
                    .primary_key(
                        Index::create()
                            .name("pk-memos_tags")
                            .col(MemosTags::MemoId)
                            .col(MemosTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-memos_tags-memo_id")
                            .from(MemosTags::Table, MemosTags::MemoId)
                            .to(Memo::Table, Memo::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-memos_tags-tag_id")
                            .from(MemosTags::Table, MemosTags::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Challenge::Table)
                    .if_not_exists()
                    .col(uuid(Challenge::Id).primary_key())
                    .col(uuid(Challenge::UserId))
                    .col(string_len(Challenge::Title, 64))
                    .col(text(Challenge::Text))
                    .col(date(Challenge::Date))
                    .col(boolean(Challenge::Archived).default(false))
                    .col(timestamp_with_time_zone_null(Challenge::AccomplishedAt))
                    .col(
                        timestamp_with_time_zone(Challenge::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Challenge::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mission-memos-user_id")
                            .from(Challenge::Table, Challenge::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(MISSION_MEMO_INDEX_NAME)
                    .table(Challenge::Table)
                    .col(Challenge::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ChallengesTags::Table)
                    .if_not_exists()
                    .col(uuid(ChallengesTags::ChallengeId))
                    .col(uuid(ChallengesTags::TagId))
                    .primary_key(
                        Index::create()
                            .name("pk-mission_memos_tags")
                            .col(ChallengesTags::ChallengeId)
                            .col(ChallengesTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mission_memos_tags-memo_id")
                            .from(ChallengesTags::Table, ChallengesTags::ChallengeId)
                            .to(Challenge::Table, Challenge::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mission_memos_tags-tag_id")
                            .from(ChallengesTags::Table, ChallengesTags::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Memo {
    Table,
    Id,
    UserId,
    Title,
    Text,
    Date,
    Archived,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum MemosTags {
    Table,
    MemoId,
    TagId,
}

#[derive(DeriveIden)]
pub enum Challenge {
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

#[derive(DeriveIden)]
pub enum ChallengesTags {
    Table,
    ChallengeId,
    TagId,
}
