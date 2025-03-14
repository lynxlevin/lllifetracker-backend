use entities::{action, ambition, challenge, challenges_tags, desired_state, tag};
use ::types::{CustomDbErr, ChallengeWithTagQueryResult};
use migration::NullOrdering::{First, Last};
use sea_orm::entity::prelude::*;
use sea_orm::{
    JoinType::LeftJoin,
    Order::{Asc, Desc},
    QueryOrder, QuerySelect,
};

pub struct ChallengeQuery;

impl ChallengeQuery {
    pub async fn find_all_with_tags_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ChallengeWithTagQueryResult>, DbErr> {
        challenge::Entity::find()
            .filter(challenge::Column::UserId.eq(user_id))
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(desired_state::Column::Name, "tag_desired_state_name")
            .column_as(action::Column::Name, "tag_action_name")
            .join_rev(LeftJoin, challenges_tags::Relation::Challenge.def())
            .join(LeftJoin, challenges_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::DesiredState.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_asc(challenge::Column::Archived)
            .order_by_with_nulls(challenge::Column::AccomplishedAt, Desc, First)
            .order_by_desc(challenge::Column::CreatedAt)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(desired_state::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .into_model::<ChallengeWithTagQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        challenge_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<challenge::Model, DbErr> {
        challenge::Entity::find_by_id(challenge_id)
            .filter(challenge::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use test_utils::{self, *};
    use chrono::Utc;

    use super::*;

    #[actix_web::test]
    async fn find_all_with_tags_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let challenge_0 = factory::challenge(user.id)
            .title("challenge_0".to_string())
            .insert(&db)
            .await?;
        let challenge_1 = factory::challenge(user.id)
            .title("challenge_1".to_string())
            .insert(&db)
            .await?;
        let archived_challenge = factory::challenge(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let accomplished_challenge = factory::challenge(user.id)
            .accomplished_at(Some(Utc::now().into()))
            .insert(&db)
            .await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (desired_state, desired_state_tag) = factory::desired_state(user.id).insert_with_tag(&db).await?;
        factory::link_challenge_tag(&db, challenge_0.id, ambition_tag.id).await?;
        factory::link_challenge_tag(&db, challenge_1.id, desired_state_tag.id).await?;
        factory::link_challenge_tag(&db, challenge_1.id, action_tag.id).await?;

        let res: Vec<ChallengeWithTagQueryResult> =
            ChallengeQuery::find_all_with_tags_by_user_id(&db, user.id).await?;

        let expected = vec![
            ChallengeWithTagQueryResult {
                id: challenge_1.id,
                title: challenge_1.title.clone(),
                text: challenge_1.text.clone(),
                date: challenge_1.date,
                archived: challenge_1.archived,
                accomplished_at: challenge_1.accomplished_at,
                created_at: challenge_1.created_at,
                updated_at: challenge_1.updated_at,
                tag_id: Some(desired_state_tag.id),
                tag_ambition_name: None,
                tag_desired_state_name: Some(desired_state.name),
                tag_action_name: None,
                tag_created_at: Some(desired_state_tag.created_at),
            },
            ChallengeWithTagQueryResult {
                id: challenge_1.id,
                title: challenge_1.title.clone(),
                text: challenge_1.text.clone(),
                date: challenge_1.date,
                archived: challenge_1.archived,
                accomplished_at: challenge_1.accomplished_at,
                created_at: challenge_1.created_at,
                updated_at: challenge_1.updated_at,
                tag_id: Some(action_tag.id),
                tag_ambition_name: None,
                tag_desired_state_name: None,
                tag_action_name: Some(action.name),
                tag_created_at: Some(action_tag.created_at),
            },
            ChallengeWithTagQueryResult {
                id: challenge_0.id,
                title: challenge_0.title.clone(),
                text: challenge_0.text.clone(),
                date: challenge_0.date,
                archived: challenge_0.archived,
                accomplished_at: challenge_0.accomplished_at,
                created_at: challenge_0.created_at,
                updated_at: challenge_0.updated_at,
                tag_id: Some(ambition_tag.id),
                tag_ambition_name: Some(ambition.name),
                tag_desired_state_name: None,
                tag_action_name: None,
                tag_created_at: Some(ambition_tag.created_at),
            },
            ChallengeWithTagQueryResult {
                id: accomplished_challenge.id,
                title: accomplished_challenge.title.clone(),
                text: accomplished_challenge.text.clone(),
                date: accomplished_challenge.date,
                archived: accomplished_challenge.archived,
                accomplished_at: accomplished_challenge.accomplished_at,
                created_at: accomplished_challenge.created_at,
                updated_at: accomplished_challenge.updated_at,
                tag_id: None,
                tag_ambition_name: None,
                tag_desired_state_name: None,
                tag_action_name: None,
                tag_created_at: None,
            },
            ChallengeWithTagQueryResult {
                id: archived_challenge.id,
                title: archived_challenge.title.clone(),
                text: archived_challenge.text.clone(),
                date: archived_challenge.date,
                archived: archived_challenge.archived,
                accomplished_at: archived_challenge.accomplished_at,
                created_at: archived_challenge.created_at,
                updated_at: archived_challenge.updated_at,
                tag_id: None,
                tag_ambition_name: None,
                tag_desired_state_name: None,
                tag_action_name: None,
                tag_created_at: None,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);
        assert_eq!(res[3], expected[3]);
        assert_eq!(res[4], expected[4]);

        Ok(())
    }
}
