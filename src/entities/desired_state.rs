//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "desired_state")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub description: Option<String>,
    pub archived: bool,
    pub ordering: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::ambitions_desired_states::Entity")]
    AmbitionsDesiredStates,
    #[sea_orm(has_many = "super::desired_states_actions::Entity")]
    DesiredStatesActions,
    #[sea_orm(has_one = "super::tag::Entity")]
    Tag,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::ambitions_desired_states::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AmbitionsDesiredStates.def()
    }
}

impl Related<super::desired_states_actions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DesiredStatesActions.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::action::Entity> for Entity {
    fn to() -> RelationDef {
        super::desired_states_actions::Relation::Action.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::desired_states_actions::Relation::DesiredState
                .def()
                .rev(),
        )
    }
}

impl Related<super::ambition::Entity> for Entity {
    fn to() -> RelationDef {
        super::ambitions_desired_states::Relation::Ambition.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::ambitions_desired_states::Relation::DesiredState
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
