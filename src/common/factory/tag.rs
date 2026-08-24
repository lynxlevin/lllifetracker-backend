use std::collections::HashMap;

use entities::{
    action, ambition, direction,
    tag::{ActiveModel, Entity, Model, TagType},
    user,
};
use sea_orm::{ActiveValue::NotSet, DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::db::Db;

pub fn tag(user_id: Uuid) -> ActiveModel {
    ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set(Some("plain_tag".to_string())),
        ambition_id: NotSet,
        direction_id: NotSet,
        action_id: NotSet,
        r#type: Set(TagType::Plain),
        ..Default::default()
    }
}

pub trait TagFactory {
    fn name(self, name: Option<String>) -> ActiveModel;
    fn ambition(self, ambition: &ambition::Model) -> ActiveModel;
    fn direction(self, direction: &direction::Model) -> ActiveModel;
    fn action(self, action: &action::Model) -> ActiveModel;
}

impl TagFactory for ActiveModel {
    fn name(mut self, name: Option<String>) -> ActiveModel {
        self.name = Set(name);
        self
    }

    fn ambition(mut self, ambition: &ambition::Model) -> ActiveModel {
        self.name = NotSet;
        self.ambition_id = Set(Some(ambition.id));
        self.direction_id = NotSet;
        self.action_id = NotSet;
        self.r#type = Set(TagType::Ambition);
        self
    }
    fn direction(mut self, direction: &direction::Model) -> ActiveModel {
        self.name = NotSet;
        self.ambition_id = NotSet;
        self.direction_id = Set(Some(direction.id));
        self.action_id = NotSet;
        self.r#type = Set(TagType::Direction);
        self
    }
    fn action(mut self, action: &action::Model) -> ActiveModel {
        self.name = NotSet;
        self.ambition_id = NotSet;
        self.direction_id = NotSet;
        self.action_id = Set(Some(action.id));
        self.r#type = Set(TagType::Action);
        self
    }
}

pub struct TagParam<'a> {
    pub name: &'a str,
    pub r#type: TagType,
    pub ambition: Option<&'a ambition::Model>,
    pub direction: Option<&'a direction::Model>,
    pub action: Option<&'a action::Model>,
}
impl Default for TagParam<'_> {
    fn default() -> Self {
        Self { name: "", r#type: TagType::Plain, ambition: None, direction: None, action: None }
    }
}

pub async fn create_tags<'a>(
    params: Vec<TagParam<'a>>,
    user: &'a user::Model,
    db: &'a Db,
) -> Result<HashMap<String, Model>, DbErr> {
    let tags = params.iter().map(|param| match param.r#type {
        TagType::Ambition => tag(user.id).ambition(param.ambition.unwrap()),
        TagType::Direction => tag(user.id).direction(param.direction.unwrap()),
        TagType::Action => tag(user.id).action(param.action.unwrap()),
        TagType::Plain => tag(user.id).name(Some(param.name.to_string())),
    });
    let tags = Entity::insert_many(tags).exec_with_returning(&db.db).await?;

    Ok(tags
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (tag, param)| {
            acc.entry(param.name.to_string()).or_insert(tag);
            acc
        }))
}
