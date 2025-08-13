use sea_orm_migration::prelude::{
    async_trait,
    sea_orm::{self, DeriveIden},
    DbErr, DeriveMigrationName, Index, MigrationTrait, SchemaManager,
};
const UNIQUE_FROM_DATE: &str = "action_goal_from_date_unique_index";
const UNIQUE_TO_DATE: &str = "action_goal_to_date_unique_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name(UNIQUE_FROM_DATE)
                    .table(ActionGoal::Table)
                    .col(ActionGoal::UserId)
                    .col(ActionGoal::ActionId)
                    .col(ActionGoal::FromDate)
                    .unique()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(UNIQUE_TO_DATE)
                    .table(ActionGoal::Table)
                    .col(ActionGoal::UserId)
                    .col(ActionGoal::ActionId)
                    .col(ActionGoal::ToDate)
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(UNIQUE_TO_DATE).to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name(UNIQUE_FROM_DATE).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ActionGoal {
    Table,
    UserId,
    ActionId,
    FromDate,
    ToDate,
}
