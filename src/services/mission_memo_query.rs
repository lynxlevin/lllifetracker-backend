use entities::{action, ambition, mission_memo, mission_memos_tags, objective, tag};
use ::types::{CustomDbErr, MissionMemoWithTagQueryResult};
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
    use test_utils::{self, *};
    use chrono::Utc;

    use super::*;

    #[actix_web::test]
    async fn find_all_with_tags_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let mission_memo_0 = factory::mission_memo(user.id)
            .title("mission_memo_0".to_string())
            .insert(&db)
            .await?;
        let mission_memo_1 = factory::mission_memo(user.id)
            .title("mission_memo_1".to_string())
            .insert(&db)
            .await?;
        let archived_mission_memo = factory::mission_memo(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let accomplished_mission_memo = factory::mission_memo(user.id)
            .accomplished_at(Some(Utc::now().into()))
            .insert(&db)
            .await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (objective, objective_tag) = factory::objective(user.id).insert_with_tag(&db).await?;
        factory::link_mission_memo_tag(&db, mission_memo_0.id, ambition_tag.id).await?;
        factory::link_mission_memo_tag(&db, mission_memo_1.id, objective_tag.id).await?;
        factory::link_mission_memo_tag(&db, mission_memo_1.id, action_tag.id).await?;

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
