use ::types::TagQueryResult;
use entities::{action, ambition, desired_state, mindset, tag};
use sea_orm::{
    sea_query::NullOrdering::Last, ColumnTrait, Condition, DbConn, DbErr, EntityTrait,
    JoinType::LeftJoin, Order::Asc, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use types::CustomDbErr;

pub struct TagQuery;

impl TagQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<TagQueryResult>, DbErr> {
        tag::Entity::find()
            .filter(tag::Column::UserId.eq(user_id))
            .filter(
                Condition::any()
                    .add(ambition::Column::Archived.eq(false))
                    .add(ambition::Column::Archived.is_null()),
            )
            .filter(
                Condition::any()
                    .add(desired_state::Column::Archived.eq(false))
                    .add(desired_state::Column::Archived.is_null()),
            )
            .filter(
                Condition::any()
                    .add(mindset::Column::Archived.eq(false))
                    .add(mindset::Column::Archived.is_null()),
            )
            .filter(
                Condition::any()
                    .add(action::Column::Archived.eq(false))
                    .add(action::Column::Archived.is_null()),
            )
            .column_as(ambition::Column::Name, "ambition_name")
            .column_as(desired_state::Column::Name, "desired_state_name")
            .column_as(mindset::Column::Name, "mindset_name")
            .column_as(action::Column::Name, "action_name")
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::DesiredState.def())
            .join(LeftJoin, tag::Relation::Mindset.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(desired_state::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(mindset::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(tag::Column::CreatedAt, Asc, Last)
            .into_model::<TagQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        tag_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<tag::Model, DbErr> {
        tag::Entity::find_by_id(tag_id)
            .filter(tag::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use common::factory::{self, *};
    use sea_orm::ActiveModelTrait;
    use test_utils;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let plain_tag = factory::tag(user.id).insert(&db).await?;
        let (_, desired_state_tag) = factory::desired_state(user.id).insert_with_tag(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (_, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (_, mindset_tag) = factory::mindset(user.id).insert_with_tag(&db).await?;
        let _archived_action = factory::action(user.id)
            .archived(true)
            .insert_with_tag(&db)
            .await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert_with_tag(&db)
            .await?;
        let _archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert_with_tag(&db)
            .await?;
        let _archived_mindset = factory::mindset(user.id)
            .archived(true)
            .insert_with_tag(&db)
            .await?;

        let res = TagQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            TagQueryResult {
                id: ambition_tag.id,
                name: None,
                ambition_name: Some("ambition".to_string()),
                desired_state_name: None,
                mindset_name: None,
                action_name: None,
                created_at: ambition_tag.created_at,
            },
            TagQueryResult {
                id: desired_state_tag.id,
                name: None,
                ambition_name: None,
                desired_state_name: Some("desired_state".to_string()),
                mindset_name: None,
                action_name: None,
                created_at: desired_state_tag.created_at,
            },
            TagQueryResult {
                id: mindset_tag.id,
                name: None,
                ambition_name: None,
                desired_state_name: None,
                mindset_name: Some("mindset".to_string()),
                action_name: None,
                created_at: mindset_tag.created_at,
            },
            TagQueryResult {
                id: action_tag.id,
                name: None,
                ambition_name: None,
                desired_state_name: None,
                mindset_name: None,
                action_name: Some("action".to_string()),
                created_at: action_tag.created_at,
            },
            TagQueryResult {
                id: plain_tag.id,
                name: plain_tag.name,
                ambition_name: None,
                desired_state_name: None,
                mindset_name: None,
                action_name: None,
                created_at: plain_tag.created_at,
            },
        ];

        assert_eq!(res.len(), expected.len());
        for i in 0..res.len() {
            dbg!(i);
            assert_eq!(res[i], expected[i]);
        }

        Ok(())
    }
}
