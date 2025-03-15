use entities::{desired_state, tag};
use chrono::Utc;
use sea_orm::{prelude::*, ActiveValue::NotSet, DbConn, DbErr, Set};
use std::future::Future;
use uuid::Uuid;

pub fn desired_state(user_id: Uuid) -> desired_state::ActiveModel {
    let now = Utc::now();
    desired_state::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        name: Set("desired_state".to_string()),
        description: Set(None),
        archived: Set(false),
        ordering: NotSet,
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait DesiredStateFactory {
    fn name(self, name: String) -> desired_state::ActiveModel;
    fn description(self, description: Option<String>) -> desired_state::ActiveModel;
    fn archived(self, archived: bool) -> desired_state::ActiveModel;
    fn ordering(self, ordering: Option<i32>) -> desired_state::ActiveModel;
    fn insert_with_tag(
        self,
        db: &DbConn,
    ) -> impl Future<Output = Result<(desired_state::Model, tag::Model), DbErr>> + Send;
}

impl DesiredStateFactory for desired_state::ActiveModel {
    fn name(mut self, name: String) -> desired_state::ActiveModel {
        self.name = Set(name);
        self
    }

    fn description(mut self, description: Option<String>) -> desired_state::ActiveModel {
        self.description = Set(description);
        self
    }

    fn archived(mut self, archived: bool) -> desired_state::ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn ordering(mut self, ordering: Option<i32>) -> desired_state::ActiveModel {
        self.ordering = Set(ordering);
        self
    }

    async fn insert_with_tag(self, db: &DbConn) -> Result<(desired_state::Model, tag::Model), DbErr> {
        let desired_state = self.insert(db).await?;
        let tag = tag::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            user_id: Set(desired_state.user_id),
            ambition_id: NotSet,
            desired_state_id: Set(Some(desired_state.id)),
            action_id: NotSet,
            created_at: Set(Utc::now().into()),
        }
        .insert(db)
        .await?;
        Ok((desired_state, tag))
    }
}
