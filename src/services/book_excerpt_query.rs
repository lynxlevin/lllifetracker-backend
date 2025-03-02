use entities::{action, ambition, book_excerpt, book_excerpts_tags, objective, tag};
use ::types::{BookExcerptWithTagQueryResult, CustomDbErr};
use migration::NullOrdering::Last;
use sea_orm::entity::prelude::*;
use sea_orm::{JoinType::LeftJoin, Order::Asc, QueryOrder, QuerySelect};

pub struct BookExcerptQuery;

impl BookExcerptQuery {
    pub async fn find_all_with_tags_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<BookExcerptWithTagQueryResult>, DbErr> {
        book_excerpt::Entity::find()
            .filter(book_excerpt::Column::UserId.eq(user_id))
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(objective::Column::Name, "tag_objective_name")
            .column_as(action::Column::Name, "tag_action_name")
            .join_rev(LeftJoin, book_excerpts_tags::Relation::BookExcerpt.def())
            .join(LeftJoin, book_excerpts_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::Objective.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_desc(book_excerpt::Column::Date)
            .order_by_desc(book_excerpt::Column::CreatedAt)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(objective::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .into_model::<BookExcerptWithTagQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        book_excerpt_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<book_excerpt::Model, DbErr> {
        book_excerpt::Entity::find_by_id(book_excerpt_id)
            .filter(book_excerpt::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_all_with_tags_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let book_excerpt_0 = factory::book_excerpt(user.id)
            .title("book_excerpt_0".to_string())
            .insert(&db)
            .await?;
        let book_excerpt_1 = factory::book_excerpt(user.id)
            .title("book_excerpt_1".to_string())
            .insert(&db)
            .await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (objective, objective_tag) = factory::objective(user.id).insert_with_tag(&db).await?;
        factory::link_book_excerpt_tag(&db, book_excerpt_0.id, ambition_tag.id).await?;
        factory::link_book_excerpt_tag(&db, book_excerpt_1.id, objective_tag.id).await?;
        factory::link_book_excerpt_tag(&db, book_excerpt_1.id, action_tag.id).await?;

        let res: Vec<BookExcerptWithTagQueryResult> =
            BookExcerptQuery::find_all_with_tags_by_user_id(&db, user.id).await?;

        let expected = vec![
            BookExcerptWithTagQueryResult {
                id: book_excerpt_1.id,
                title: book_excerpt_1.title.clone(),
                page_number: book_excerpt_1.page_number,
                text: book_excerpt_1.text.clone(),
                date: book_excerpt_1.date,
                created_at: book_excerpt_1.created_at,
                updated_at: book_excerpt_1.updated_at,
                tag_id: Some(objective_tag.id),
                tag_ambition_name: None,
                tag_objective_name: Some(objective.name),
                tag_action_name: None,
                tag_created_at: Some(objective_tag.created_at),
            },
            BookExcerptWithTagQueryResult {
                id: book_excerpt_1.id,
                title: book_excerpt_1.title.clone(),
                page_number: book_excerpt_1.page_number,
                text: book_excerpt_1.text.clone(),
                date: book_excerpt_1.date,
                created_at: book_excerpt_1.created_at,
                updated_at: book_excerpt_1.updated_at,
                tag_id: Some(action_tag.id),
                tag_ambition_name: None,
                tag_objective_name: None,
                tag_action_name: Some(action.name),
                tag_created_at: Some(action_tag.created_at),
            },
            BookExcerptWithTagQueryResult {
                id: book_excerpt_0.id,
                title: book_excerpt_0.title.clone(),
                page_number: book_excerpt_0.page_number,
                text: book_excerpt_0.text.clone(),
                date: book_excerpt_0.date,
                created_at: book_excerpt_0.created_at,
                updated_at: book_excerpt_0.updated_at,
                tag_id: Some(ambition_tag.id),
                tag_ambition_name: Some(ambition.name),
                tag_objective_name: None,
                tag_action_name: None,
                tag_created_at: Some(ambition_tag.created_at),
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);

        Ok(())
    }
}
