use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        Alias, DbErr, DeriveMigrationName, Expr, ForeignKey, ForeignKeyAction, MigrationTrait,
        Query, SchemaManager, Table, TableForeignKey,
    },
    schema::{
        boolean, integer_null, string, string_null, timestamp_with_time_zone, uuid, uuid_null,
    },
};

const TAGS_MINDSET_FOREIGN_KEY_NAME: &str = "fk-tags-mindset_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(Tag::Table)
                    .and_where(Expr::col((Tag::Table, Tag::MindsetId)).is_not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .drop_foreign_key(Alias::new(TAGS_MINDSET_FOREIGN_KEY_NAME))
                    .drop_column(Tag::MindsetId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Mindset::Table).to_owned())
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Mindset::Table)
                    .if_not_exists()
                    .col(uuid(Mindset::Id).primary_key())
                    .col(uuid(Mindset::UserId))
                    .col(string(Mindset::Name))
                    .col(string_null(Mindset::Description))
                    .col(boolean(Mindset::Archived).default(false))
                    .col(integer_null(Mindset::Ordering))
                    .col(
                        timestamp_with_time_zone(Mindset::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Mindset::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mindsets-user_id")
                            .from(Mindset::Table, Mindset::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .add_column_if_not_exists(uuid_null(Tag::MindsetId).unique_key())
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(TAGS_MINDSET_FOREIGN_KEY_NAME)
                            .from_tbl(Tag::Table)
                            .from_col(Tag::MindsetId)
                            .to_tbl(Mindset::Table)
                            .to_col(Mindset::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Mindset {
    Table,
    Id,
    UserId,
    Name,
    Description,
    Archived,
    Ordering,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    MindsetId,
}
