use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use sea_orm_migration::prelude::*;
use uuid::Uuid;

use crate::{
    m20240722_000001_create_users_table::User, m20240927_000001_create_ambitions_table::Ambition,
    m20240927_000002_create_objectives_table::Objective,
    m20240927_000003_create_actions_table::Action,
    m20240927_000004_create_ambitions_objectives_table::AmbitionsObjectives,
    m20240927_000005_create_objectives_actions_table::ObjectivesActions,
    m20240927_000006_create_tags_table::Tag, m20240927_000007_create_records_table::Record,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut stmts: Vec<InsertStatement> = vec![];

        let user_1_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(User::Table)
                .columns([
                    User::Id,
                    User::Email,
                    User::Password,
                    User::FirstName,
                    User::LastName,
                    User::IsActive,
                ])
                .values_panic([
                    user_1_id.into(),
                    "test@test.com".into(),
                    hash("password".as_bytes()).await.into(),
                    "Lynx".into(),
                    "Levin".into(),
                    true.into(),
                ])
                .to_owned(),
        );

        let ambition_1_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(Ambition::Table)
                .columns([Ambition::Id, Ambition::UserId, Ambition::Name])
                .values_panic([ambition_1_id.into(), user_1_id.into(), "My ambition".into()])
                .to_owned(),
        );
        let ambition_1_tag_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(Tag::Table)
                .columns([Tag::Id, Tag::UserId, Tag::AmbitionId])
                .values_panic([
                    ambition_1_tag_id.into(),
                    user_1_id.into(),
                    ambition_1_id.into(),
                ])
                .to_owned(),
        );

        let objective_1_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(Objective::Table)
                .columns([Objective::Id, Objective::UserId, Objective::Name])
                .values_panic([
                    objective_1_id.into(),
                    user_1_id.into(),
                    "My objective".into(),
                ])
                .to_owned(),
        );
        let objective_1_tag_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(Tag::Table)
                .columns([Tag::Id, Tag::UserId, Tag::ObjectiveId])
                .values_panic([
                    objective_1_tag_id.into(),
                    user_1_id.into(),
                    objective_1_id.into(),
                ])
                .to_owned(),
        );

        let action_1_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(Action::Table)
                .columns([Action::Id, Action::UserId, Action::Name])
                .values_panic([action_1_id.into(), user_1_id.into(), "My action".into()])
                .to_owned(),
        );
        let action_1_tag_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(Tag::Table)
                .columns([Tag::Id, Tag::UserId, Tag::ActionId])
                .values_panic([action_1_tag_id.into(), user_1_id.into(), action_1_id.into()])
                .to_owned(),
        );

        stmts.push(
            Query::insert()
                .into_table(AmbitionsObjectives::Table)
                .columns([
                    AmbitionsObjectives::AmbitionId,
                    AmbitionsObjectives::ObjectiveId,
                ])
                .values_panic([ambition_1_id.into(), objective_1_id.into()])
                .to_owned(),
        );
        stmts.push(
            Query::insert()
                .into_table(ObjectivesActions::Table)
                .columns([ObjectivesActions::ObjectiveId, ObjectivesActions::ActionId])
                .values_panic([objective_1_id.into(), action_1_id.into()])
                .to_owned(),
        );

        let record_1_id = Uuid::new_v4();
        stmts.push(
            Query::insert()
                .into_table(Record::Table)
                .columns([
                    Record::Id,
                    Record::UserId,
                    Record::TagId,
                    Record::ActionName,
                ])
                .values_panic([
                    record_1_id.into(),
                    user_1_id.into(),
                    action_1_tag_id.into(),
                    "My action".into(),
                ])
                .to_owned(),
        );

        for stmt in stmts {
            manager.exec_stmt(stmt).await?;
        }

        Ok(())
    }
}

async fn hash(password: &[u8]) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password, &salt)
        .expect("Unable to hash password.")
        .to_string()
}
