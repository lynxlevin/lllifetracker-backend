use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20240722_000001_create_users_table::User, m20240927_000001_create_ambitions_table::Ambition,
    m20240927_000002_create_objectives_table::Objective,
    m20240927_000003_create_actions_table::Action,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(uuid(Tag::Id).primary_key())
                    .col(uuid(Tag::UserId))
                    .col(uuid_null(Tag::AmbitionId).unique_key())
                    .col(uuid_null(Tag::ObjectiveId).unique_key())
                    .col(uuid_null(Tag::ActionId).unique_key())
                    .col(
                        timestamp_with_time_zone(Tag::CreatedAt).default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("tags_user_fkey")
                    .from(Tag::Table, Tag::UserId)
                    .to(User::Table, User::Id)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("tag_ambition_fkey")
                    .from(Tag::Table, Tag::AmbitionId)
                    .to(Ambition::Table, Ambition::Id)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("tag_objective_fkey")
                    .from(Tag::Table, Tag::ObjectiveId)
                    .to(Objective::Table, Objective::Id)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                sea_query::ForeignKey::create()
                    .name("tag_action_fkey")
                    .from(Tag::Table, Tag::ActionId)
                    .to(Action::Table, Action::Id)
                    .on_delete(sea_query::ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("tag_action_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("tag_objective_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("tag_ambition_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_foreign_key(
                sea_query::ForeignKey::drop()
                    .name("tags_user_fkey")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Id,
    UserId,
    AmbitionId,
    ObjectiveId,
    ActionId,
    CreatedAt,
}
