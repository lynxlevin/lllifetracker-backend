use std::future::Future;

use chrono::Utc;
use sea_orm::{
    sea_query::{IntoCondition, NullOrdering::Last},
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel,
    JoinType::LeftJoin,
    ModelTrait, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select, Set,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use entities::{
    action::{ActiveModel, Column, Entity, Model, Relation},
    action_goal,
    sea_orm_active_enums::{ActionTrackType, TagType},
    tag, user,
};

#[derive(Clone)]
pub struct ActionAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> ActionAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait ActionJoin {
    fn join_active_goal(self) -> Self;
}

impl ActionJoin for ActionAdapter<'_> {
    fn join_active_goal(mut self) -> Self {
        self.query = self.query.join(
            LeftJoin,
            Relation::ActionGoal.def().on_condition(|_left, _right| {
                action_goal::Column::ToDate.is_null().into_condition()
            }),
        );
        self
    }
}

pub trait ActionFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_eq_archived(self, archived: bool) -> Self;
    fn filter_in_ids(self, ids: Vec<Uuid>) -> Self;
}

impl ActionFilter for ActionAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }

    fn filter_eq_archived(mut self, archived: bool) -> Self {
        self.query = self.query.filter(Column::Archived.eq(archived));
        self
    }

    fn filter_in_ids(mut self, ids: Vec<Uuid>) -> Self {
        self.query = self.query.filter(Column::Id.is_in(ids));
        self
    }
}

pub trait ActionOrder {
    fn order_by_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_created_at(self, order: Order) -> Self;
}

impl ActionOrder for ActionAdapter<'_> {
    fn order_by_ordering_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(Column::Ordering, order, Last);
        self
    }

    fn order_by_created_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::CreatedAt, order);
        self
    }
}

pub trait ActionQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    fn get_all_with_goal(
        self,
    ) -> impl Future<Output = Result<Vec<(Model, Option<action_goal::Model>)>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl ActionQuery for ActionAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    async fn get_all_with_goal(self) -> Result<Vec<(Model, Option<action_goal::Model>)>, DbErr> {
        self.query
            .select_also(action_goal::Entity)
            .all(self.db)
            .await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateActionParams {
    pub name: String,
    pub description: Option<String>,
    pub track_type: ActionTrackType,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateActionParams {
    pub name: String,
    pub description: Option<String>,
    pub trackable: Option<bool>,
    pub color: Option<String>,
}

pub trait ActionMutation {
    fn create_with_tag(
        self,
        params: CreateActionParams,
    ) -> impl Future<Output = Result<Model, TransactionError<DbErr>>>;
    fn update(
        self,
        action: Model,
        params: UpdateActionParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn convert_track_type(
        self,
        action: Model,
        track_type: ActionTrackType,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn archive(self, action: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn unarchive(self, action: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn bulk_update_ordering(
        self,
        actions: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn delete(self, action: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl ActionMutation for ActionAdapter<'_> {
    async fn create_with_tag(
        self,
        params: CreateActionParams,
    ) -> Result<Model, TransactionError<DbErr>> {
        self.db
            .transaction::<_, Model, DbErr>(|txn| {
                Box::pin(async move {
                    let action_id = uuid::Uuid::now_v7();
                    let created_action = ActiveModel {
                        id: Set(action_id),
                        user_id: Set(params.user_id),
                        name: Set(params.name.to_owned()),
                        description: Set(params.description.to_owned()),
                        track_type: Set(params.track_type),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;
                    tag::ActiveModel {
                        id: Set(uuid::Uuid::now_v7()),
                        user_id: Set(params.user_id),
                        action_id: Set(Some(action_id)),
                        r#type: Set(TagType::Action),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    Ok(created_action)
                })
            })
            .await
    }

    async fn update(self, action: Model, params: UpdateActionParams) -> Result<Model, DbErr> {
        let mut action = action.into_active_model();
        action.name = Set(params.name);
        action.description = Set(params.description);
        if let Some(trackable) = params.trackable {
            action.trackable = Set(trackable);
        }
        if let Some(color) = params.color {
            action.color = Set(color);
        }
        action.updated_at = Set(Utc::now().into());
        action.update(self.db).await
    }

    async fn convert_track_type(
        self,
        action: Model,
        track_type: ActionTrackType,
    ) -> Result<Model, DbErr> {
        let mut action = action.into_active_model();
        action.track_type = Set(track_type);
        action.updated_at = Set(Utc::now().into());
        action.update(self.db).await
    }

    async fn archive(self, action: Model) -> Result<Model, DbErr> {
        let mut action = action.into_active_model();
        action.archived = Set(true);
        action.updated_at = Set(Utc::now().into());
        action.update(self.db).await
    }

    async fn unarchive(self, action: Model) -> Result<Model, DbErr> {
        let mut action = action.into_active_model();
        action.archived = Set(false);
        action.updated_at = Set(Utc::now().into());
        action.update(self.db).await
    }

    async fn bulk_update_ordering(
        self,
        actions: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> Result<(), DbErr> {
        for action in actions {
            let order = &ordering.iter().position(|id| id == &action.id);
            if let Some(order) = order {
                let mut action = action.into_active_model();
                action.ordering = Set(Some((order + 1) as i32));
                action.update(self.db).await?;
            }
        }
        Ok(())
    }

    async fn delete(self, action: Model) -> Result<(), DbErr> {
        action.delete(self.db).await.map(|_| ())
    }
}
