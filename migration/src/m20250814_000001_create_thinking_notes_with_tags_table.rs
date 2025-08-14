use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, Expr, ForeignKey, ForeignKeyAction, Index, MigrationTrait,
        SchemaManager, Table,
    },
    schema::{
        string_null, text_null, timestamp_with_time_zone, timestamp_with_time_zone_null, uuid,
    },
};

use crate::m20240722_000001_create_users_table::User;

const INDEX_NAME: &str = "thinking_notes_user_id_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ThinkingNote::Table)
                    .if_not_exists()
                    .col(uuid(ThinkingNote::Id).primary_key())
                    .col(uuid(ThinkingNote::UserId))
                    .col(string_null(ThinkingNote::Question))
                    .col(text_null(ThinkingNote::Thought))
                    .col(string_null(ThinkingNote::Answer))
                    .col(
                        timestamp_with_time_zone_null(ThinkingNote::ResolvedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone_null(ThinkingNote::ArchivedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ThinkingNote::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ThinkingNote::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-thinking_notes-user_id")
                            .from(ThinkingNote::Table, ThinkingNote::UserId)
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
                    .table(ThinkingNote::Table)
                    .col(ThinkingNote::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(ThinkingNoteTags::Table)
                    .if_not_exists()
                    .col(uuid(ThinkingNoteTags::ThinkingNoteId))
                    .col(uuid(ThinkingNoteTags::TagId))
                    .primary_key(
                        Index::create()
                            .name("pk-thinking_note_tags")
                            .col(ThinkingNoteTags::ThinkingNoteId)
                            .col(ThinkingNoteTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-thinking_note_tags-thinking_note_id")
                            .from(ThinkingNoteTags::Table, ThinkingNoteTags::ThinkingNoteId)
                            .to(ThinkingNote::Table, ThinkingNote::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-thinking_note_tags-tag_id")
                            .from(ThinkingNoteTags::Table, ThinkingNoteTags::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ThinkingNoteTags::Table).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ThinkingNote::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ThinkingNote {
    Table,
    Id,
    UserId,
    Question,
    Thought,
    Answer,
    ResolvedAt,
    ArchivedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum ThinkingNoteTags {
    Table,
    ThinkingNoteId,
    TagId,
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Id,
}
