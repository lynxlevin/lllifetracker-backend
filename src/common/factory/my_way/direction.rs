use chrono::Utc;
use entities::{direction, sea_orm_active_enums::TagType, tag};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, Set};
use std::future::Future;
use uuid::Uuid;

pub fn direction(user_id: Uuid) -> direction::ActiveModel {
    let now = Utc::now();
    direction::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set("direction".to_string()),
        description: Set(None),
        archived: Set(false),
        ordering: NotSet,
        category_id: NotSet,
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait DirectionFactory {
    fn name(self, name: String) -> direction::ActiveModel;
    fn description(self, description: Option<String>) -> direction::ActiveModel;
    fn archived(self, archived: bool) -> direction::ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> direction::ActiveModel;
    fn category_id(self, category_id: Option<Uuid>) -> direction::ActiveModel;
    fn insert_with_tag(
        self,
        db: &DbConn,
    ) -> impl Future<Output = Result<(direction::Model, tag::Model), DbErr>> + Send;
}

impl DirectionFactory for direction::ActiveModel {
    fn name(mut self, name: String) -> direction::ActiveModel {
        self.name = Set(name);
        self
    }

    fn description(mut self, description: Option<String>) -> direction::ActiveModel {
        self.description = Set(description);
        self
    }

    fn archived(mut self, archived: bool) -> direction::ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> direction::ActiveModel {
        self.ordering = Set(ordering);
        self
    }

    fn category_id(mut self, category_id: Option<Uuid>) -> direction::ActiveModel {
        self.category_id = Set(category_id);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(direction::Model, tag::Model), DbErr> {
        let direction = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(direction.user_id),
            direction_id: Set(Some(direction.id)),
            r#type: Set(TagType::Direction),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok((direction, tag))
    }
}
