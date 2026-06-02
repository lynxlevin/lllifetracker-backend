use chrono::Utc;
use entities::{
    ambition::{ActiveModel, Entity, Model},
    sea_orm_active_enums::TagType,
    tag, user,
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, EntityTrait, Set};
use std::{collections::HashMap, future::Future};
use uuid::Uuid;

pub fn ambition(user_id: Uuid) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set("ambition".to_string()),
        description: Set(None),
        archived: Set(false),
        ordering: NotSet,
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait AmbitionFactory {
    fn name(self, name: String) -> ActiveModel;
    fn description(self, description: Option<String>) -> ActiveModel;
    fn archived(self, archived: bool) -> ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> ActiveModel;
    fn insert_with_tag(self, db: &DbConn) -> impl Future<Output = Result<(Model, tag::Model), DbErr>> + Send;
}

impl AmbitionFactory for ActiveModel {
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

    async fn insert_with_tag(self, db: &DbConn) -> Result<(Model, tag::Model), DbErr> {
        let ambition = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(ambition.user_id),
            ambition_id: Set(Some(ambition.id)),
            r#type: Set(TagType::Ambition),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok((ambition, tag))
    }
}

#[derive(Default)]
pub struct AmbitionParam<'a> {
    pub name: &'a str,
    pub archived: bool,
    pub ordering: Option<i32>,
}

pub async fn create_ambitions<'a>(
    params: Vec<AmbitionParam<'a>>,
    user: &'a user::Model,
    db: &'a DbConn,
) -> Result<HashMap<String, Model>, DbErr> {
    let ambitions = params.iter().map(|param| {
        ambition(user.id)
            .name(param.name.to_string())
            .archived(param.archived)
            .ordering(param.ordering)
    });
    let ambitions = Entity::insert_many(ambitions).exec_with_returning_many(db).await?;

    Ok(ambitions
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (ambition, param)| {
            acc.entry(param.name.to_string()).or_insert(ambition);
            acc
        }))
}
