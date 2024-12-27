use crate::entities::{action, ambition, mission_memo, mission_memos_tags, objective, tag};
use crate::types::{CustomDbErr, MissionMemoWithTagQueryResult};
use migration::NullOrdering::{First, Last};
use sea_orm::entity::prelude::*;
use sea_orm::{
    JoinType::LeftJoin,
    Order::{Asc, Desc},
    QueryOrder, QuerySelect,
};

pub struct MissionMemoQuery;

impl MissionMemoQuery {
    pub async fn find_all_with_tags_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<MissionMemoWithTagQueryResult>, DbErr> {
        mission_memo::Entity::find()
            .filter(mission_memo::Column::UserId.eq(user_id))
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(objective::Column::Name, "tag_objective_name")
            .column_as(action::Column::Name, "tag_action_name")
            .join_rev(LeftJoin, mission_memos_tags::Relation::MissionMemo.def())
            .join(LeftJoin, mission_memos_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::Objective.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_asc(mission_memo::Column::Archived)
            .order_by_with_nulls(mission_memo::Column::AccomplishedAt, Desc, First)
            .order_by_desc(mission_memo::Column::CreatedAt)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(objective::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .into_model::<MissionMemoWithTagQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        mission_memo_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<mission_memo::Model, DbErr> {
        mission_memo::Entity::find_by_id(mission_memo_id)
            .filter(mission_memo::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{self, factory};
    use chrono::Utc;
    use sea_orm::ActiveValue::Set;

    use super::*;

    #[actix_web::test]
    async fn find_all_with_tags_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let mission_memo_0 =
            test_utils::seed::create_mission_memo(&db, "mission_memo_0".to_string(), user.id)
                .await?;
        let mission_memo_1 =
            test_utils::seed::create_mission_memo(&db, "mission_memo_1".to_string(), user.id)
                .await?;
        let mut archived_mission_memo: mission_memo::ActiveModel =
            test_utils::seed::create_mission_memo(
                &db,
                "archived_mission_memo".to_string(),
                user.id,
            )
            .await?
            .into();
        archived_mission_memo.archived = Set(true);
        let archived_mission_memo = archived_mission_memo.update(&db).await?;
        let mut accomplished_mission_memo: mission_memo::ActiveModel =
            test_utils::seed::create_mission_memo(
                &db,
                "accomplished_mission_memo".to_string(),
                user.id,
            )
            .await?
            .into();
        accomplished_mission_memo.accomplished_at = Set(Some(Utc::now().into()));
        let accomplished_mission_memo = accomplished_mission_memo.update(&db).await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (objective, objective_tag) =
            test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), None, user.id)
                .await?;
        mission_memos_tags::ActiveModel {
            mission_memo_id: Set(mission_memo_0.id),
            tag_id: Set(ambition_tag.id),
        }
        .insert(&db)
        .await?;
        mission_memos_tags::ActiveModel {
            mission_memo_id: Set(mission_memo_1.id),
            tag_id: Set(objective_tag.id),
        }
        .insert(&db)
        .await?;
        mission_memos_tags::ActiveModel {
            mission_memo_id: Set(mission_memo_1.id),
            tag_id: Set(action_tag.id),
        }
        .insert(&db)
        .await?;

        let res: Vec<MissionMemoWithTagQueryResult> =
            MissionMemoQuery::find_all_with_tags_by_user_id(&db, user.id).await?;

        let expected = vec![
            MissionMemoWithTagQueryResult {
                id: mission_memo_1.id,
                title: mission_memo_1.title.clone(),
                text: mission_memo_1.text.clone(),
                date: mission_memo_1.date,
                archived: mission_memo_1.archived,
                accomplished_at: mission_memo_1.accomplished_at,
                created_at: mission_memo_1.created_at,
                updated_at: mission_memo_1.updated_at,
                tag_id: Some(objective_tag.id),
                tag_ambition_name: None,
                tag_objective_name: Some(objective.name),
                tag_action_name: None,
                tag_created_at: Some(objective_tag.created_at),
            },
            MissionMemoWithTagQueryResult {
                id: mission_memo_1.id,
                title: mission_memo_1.title.clone(),
                text: mission_memo_1.text.clone(),
                date: mission_memo_1.date,
                archived: mission_memo_1.archived,
                accomplished_at: mission_memo_1.accomplished_at,
                created_at: mission_memo_1.created_at,
                updated_at: mission_memo_1.updated_at,
                tag_id: Some(action_tag.id),
                tag_ambition_name: None,
                tag_objective_name: None,
                tag_action_name: Some(action.name),
                tag_created_at: Some(action_tag.created_at),
            },
            MissionMemoWithTagQueryResult {
                id: mission_memo_0.id,
                title: mission_memo_0.title.clone(),
                text: mission_memo_0.text.clone(),
                date: mission_memo_0.date,
                archived: mission_memo_0.archived,
                accomplished_at: mission_memo_0.accomplished_at,
                created_at: mission_memo_0.created_at,
                updated_at: mission_memo_0.updated_at,
                tag_id: Some(ambition_tag.id),
                tag_ambition_name: Some(ambition.name),
                tag_objective_name: None,
                tag_action_name: None,
                tag_created_at: Some(ambition_tag.created_at),
            },
            MissionMemoWithTagQueryResult {
                id: accomplished_mission_memo.id,
                title: accomplished_mission_memo.title.clone(),
                text: accomplished_mission_memo.text.clone(),
                date: accomplished_mission_memo.date,
                archived: accomplished_mission_memo.archived,
                accomplished_at: accomplished_mission_memo.accomplished_at,
                created_at: accomplished_mission_memo.created_at,
                updated_at: accomplished_mission_memo.updated_at,
                tag_id: None,
                tag_ambition_name: None,
                tag_objective_name: None,
                tag_action_name: None,
                tag_created_at: None,
            },
            MissionMemoWithTagQueryResult {
                id: archived_mission_memo.id,
                title: archived_mission_memo.title.clone(),
                text: archived_mission_memo.text.clone(),
                date: archived_mission_memo.date,
                archived: archived_mission_memo.archived,
                accomplished_at: archived_mission_memo.accomplished_at,
                created_at: archived_mission_memo.created_at,
                updated_at: archived_mission_memo.updated_at,
                tag_id: None,
                tag_ambition_name: None,
                tag_objective_name: None,
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
