use sea_orm_migration::{
    prelude::{
        async_trait,
        sea_orm::{self, DeriveIden},
        DbErr, DeriveMigrationName, ForeignKey, ForeignKeyAction, Index, MigrationTrait,
        SchemaManager, Table,
    },
    schema::{date, date_null, integer_null, uuid},
};

const INDEX_USER_ID: &str = "action_goal_user_id_index";
const INDEX_FROM_DATE: &str = "action_goal_from_date_index";
const INDEX_TO_DATE: &str = "action_goal_to_date_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

// This table will have few writes and many reads, so create many indexes.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ActionGoal::Table)
                    .if_not_exists()
                    .col(uuid(ActionGoal::Id).primary_key())
                    .col(uuid(ActionGoal::UserId))
                    .col(uuid(ActionGoal::ActionId))
                    .col(date(ActionGoal::FromDate))
                    .col(date_null(ActionGoal::ToDate))
                    .col(integer_null(ActionGoal::DurationSeconds))
                    .col(integer_null(ActionGoal::Count))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-action_goal-user_id")
                            .from(ActionGoal::Table, ActionGoal::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-action_goal-action_id")
                            .from(ActionGoal::Table, ActionGoal::UserId)
                            .to(Action::Table, Action::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(INDEX_USER_ID)
                    .table(ActionGoal::Table)
                    .col(ActionGoal::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(INDEX_FROM_DATE)
                    .table(ActionGoal::Table)
                    .col(ActionGoal::FromDate)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(INDEX_TO_DATE)
                    .table(ActionGoal::Table)
                    .col(ActionGoal::ToDate)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(INDEX_TO_DATE).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(INDEX_FROM_DATE).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(INDEX_USER_ID).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ActionGoal::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ActionGoal {
    Table,
    Id,
    UserId,
    ActionId,
    FromDate,
    ToDate,
    DurationSeconds,
    Count,
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum Action {
    Table,
    Id,
}
