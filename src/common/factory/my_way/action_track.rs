use std::collections::HashMap;

use chrono::{DateTime, Duration, FixedOffset, SubsecRound, Utc};
use entities::{
    action_track::{ActiveModel, Entity, Model},
    user,
};
use sea_orm::{ActiveValue::NotSet, DbErr, EntityTrait, Set};
use uuid::Uuid;

use crate::db::Db;

pub fn action_track(user_id: Uuid) -> ActiveModel {
    ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        action_id: NotSet,
        started_at: Set(Utc::now().trunc_subsecs(0).into()),
        ended_at: NotSet,
        duration: NotSet,
    }
}

pub trait ActionTrackFactory {
    fn action_id(self, action_id: Uuid) -> ActiveModel;
    fn duration(self, duration: Option<i64>) -> ActiveModel;
    fn started_at(self, started_at: chrono::DateTime<chrono::FixedOffset>) -> ActiveModel;
}

impl ActionTrackFactory for ActiveModel {
    fn action_id(mut self, action_id: Uuid) -> ActiveModel {
        self.action_id = Set(action_id);
        self
    }

    fn duration(mut self, duration: Option<i64>) -> ActiveModel {
        self.duration = Set(duration);
        match duration {
            Some(duration) => {
                self.ended_at = Set(Some(
                    (self.started_at.clone().unwrap() + Duration::seconds(duration)).into(),
                ));
            }
            None => self.ended_at = Set(None),
        }
        self
    }

    fn started_at(mut self, started_at: chrono::DateTime<chrono::FixedOffset>) -> ActiveModel {
        self.started_at = Set(started_at);
        if self.duration == NotSet {
            return self;
        }
        if let Some(duration) = self.duration.clone().unwrap() {
            self.ended_at = Set((started_at + Duration::seconds(duration)).into());
        }
        self
    }
}

#[derive(Default)]
pub struct ActionTrackParam<'a> {
    pub name: &'a str,
    pub action_id: Uuid,
    pub started_at: DateTime<FixedOffset>,
    pub duration: Option<i64>,
}

pub async fn create_action_tracks<'a>(
    params: Vec<ActionTrackParam<'a>>,
    user: &'a user::Model,
    db: &'a Db,
) -> Result<HashMap<String, Model>, DbErr> {
    let action_tracks = params.iter().map(|param| {
        action_track(user.id)
            .action_id(param.action_id)
            .started_at(param.started_at)
            .duration(param.duration)
    });
    let action_tracks = Entity::insert_many(action_tracks).exec_with_returning(&db.db).await?;

    Ok(action_tracks
        .into_iter()
        .zip(params)
        .fold(HashMap::new(), |mut acc, (action_track, param)| {
            acc.entry(param.name.to_string()).or_insert(action_track);
            acc
        }))
}
