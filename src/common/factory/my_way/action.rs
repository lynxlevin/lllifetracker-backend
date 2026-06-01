use chrono::Utc;
use entities::{
    action::{ActiveModel, Entity, Model},
    sea_orm_active_enums::{ActionTrackType, TagType},
    tag, user,
};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, EntityTrait, Set};
use std::{collections::HashMap, future::Future};
use uuid::Uuid;

pub fn action(user_id: Uuid) -> ActiveModel {
    let now = Utc::now();
    ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        name: Set("action".to_string()),
        discipline: Set(None),
        memo: Set(None),
        archived: Set(false),
        ordering: NotSet,
        color: Set("#212121".to_string()),
        track_type: Set(ActionTrackType::TimeSpan),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait ActionFactory {
    fn name(self, name: String) -> ActiveModel;
    fn discipline(self, discipline: Option<String>) -> ActiveModel;
    fn archived(self, archived: bool) -> ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> ActiveModel;
    fn track_type(self, track_type: ActionTrackType) -> ActiveModel;
    fn insert_with_tag(self, db: &DbConn) -> impl Future<Output = Result<(Model, tag::Model), DbErr>> + Send;
}

impl ActionFactory for ActiveModel {
    fn name(mut self, name: String) -> ActiveModel {
        self.name = Set(name);
        self
    }

    fn discipline(mut self, discipline: Option<String>) -> ActiveModel {
        self.discipline = Set(discipline);
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

    fn track_type(mut self, track_type: ActionTrackType) -> ActiveModel {
        self.track_type = Set(track_type);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(Model, tag::Model), DbErr> {
        let action = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(action.user_id),
            action_id: Set(Some(action.id)),
            r#type: Set(TagType::Action),
            ..Default::default()
        }
        .insert(db)
        .await?;
        Ok((action, tag))
    }
}

#[derive(Default)]
pub struct ActionParam {
    pub name: String,
    pub archived: bool,
    pub ordering: Option<i32>,
    pub track_type: Option<ActionTrackType>,
}

pub async fn create_actions<'a>(
    params: Vec<ActionParam>,
    user: &'a user::Model,
    db: &'a DbConn,
) -> Result<HashMap<String, Model>, DbErr> {
    let actions = params.iter().map(|param| {
        action(user.id)
            .name(param.name.clone())
            .archived(param.archived)
            .ordering(param.ordering)
            .track_type(param.track_type.clone().or(Some(ActionTrackType::TimeSpan)).unwrap())
    });
    let actions = Entity::insert_many(actions).exec_with_returning_many(db).await?;

    Ok(actions
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (action, param)| {
            acc.entry(param.name.to_string()).or_insert(action);
            acc
        }))
}
