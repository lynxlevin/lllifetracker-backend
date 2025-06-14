use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        Alias, DbErr, DeriveMigrationName, ForeignKey, ForeignKeyAction, Index, MigrationTrait,
        SchemaManager, Table, TableForeignKey,
    },
    schema::{integer_null, string, uuid, uuid_null},
};

const INDEX_NAME: &str = "desired_state_category_user_id_index";
const CATEGORY_ID_INDEX_NAME: &str = "desired_state_category_id_user_id_index";
const DESIRED_STATE_CATEGORY_FOREIGN_KEY_NAME: &str = "fk-desired_state-category_id";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DesiredStateCategory::Table)
                    .if_not_exists()
                    .col(uuid(DesiredStateCategory::Id).primary_key())
                    .col(uuid(DesiredStateCategory::UserId))
                    .col(string(DesiredStateCategory::Name))
                    .col(integer_null(DesiredStateCategory::Ordering))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-desired_state_category-user_id")
                            .from(DesiredStateCategory::Table, DesiredStateCategory::UserId)
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
                    .table(DesiredStateCategory::Table)
                    .col(DesiredStateCategory::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(DesiredState::Table)
                    .add_column_if_not_exists(uuid_null(DesiredState::CategoryId))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name(DESIRED_STATE_CATEGORY_FOREIGN_KEY_NAME)
                            .from_tbl(DesiredState::Table)
                            .from_col(DesiredState::CategoryId)
                            .to_tbl(DesiredStateCategory::Table)
                            .to_col(DesiredStateCategory::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(CATEGORY_ID_INDEX_NAME)
                    .table(DesiredState::Table)
                    .col(DesiredState::CategoryId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(CATEGORY_ID_INDEX_NAME).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(DesiredState::Table)
                    .drop_foreign_key(Alias::new(DESIRED_STATE_CATEGORY_FOREIGN_KEY_NAME))
                    .drop_column(DesiredState::CategoryId)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(Index::drop().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(DesiredStateCategory::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum DesiredStateCategory {
    Table,
    Id,
    UserId,
    Name,
    Ordering,
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum DesiredState {
    Table,
    CategoryId,
}
