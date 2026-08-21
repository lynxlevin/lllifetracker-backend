use chrono::Utc;
use entities::{
    direction::{ActiveModel, Entity, Model},
    sea_orm_active_enums::TagType,
    tag, user,
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, EntityTrait, Set};
use std::{collections::HashMap, future::Future};
use uuid::Uuid;

pub fn direction(user_id: Uuid) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
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
    fn name(self, name: String) -> ActiveModel;
    fn description(self, description: Option<String>) -> ActiveModel;
    fn archived(self, archived: bool) -> ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> ActiveModel;
    fn category_id(self, category_id: Option<Uuid>) -> ActiveModel;
    fn insert_with_tag(self, db: &DbConn) -> impl Future<Output = Result<(Model, tag::Model), DbErr>> + Send;
}

impl DirectionFactory for ActiveModel {
    fn name(mut self, name: String) -> ActiveModel {
        self.name = Set(name);
        self
    }

    fn description(mut self, description: Option<String>) -> ActiveModel {
        self.description = Set(description);
        self
    }

    fn archived(mut self, archived: bool) -> ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> ActiveModel {
        self.ordering = Set(ordering);
        self
    }

    fn category_id(mut self, category_id: Option<Uuid>) -> ActiveModel {
        self.category_id = Set(category_id);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(Model, tag::Model), DbErr> {
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

#[derive(Default)]
pub struct DirectionParam<'a> {
    pub name: &'a str,
    pub archived: bool,
    pub ordering: Option<i32>,
    pub category_id: Option<Uuid>,
}

pub async fn create_directions<'a>(
    params: Vec<DirectionParam<'a>>,
    user: &'a user::Model,
    db: &'a DbConn,
) -> Result<HashMap<String, Model>, DbErr> {
    let directions = params.iter().map(|param| {
        direction(user.id)
            .name(param.name.to_string())
            .archived(param.archived)
            .ordering(param.ordering)
            .category_id(param.category_id)
    });
    let directions = Entity::insert_many(directions).exec_with_returning(db).await?;

    Ok(directions
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (direction, param)| {
            acc.entry(param.name.to_string()).or_insert(direction);
            acc
        }))
}
