//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use super::sea_orm_active_enums::TimezoneEnum;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub timezone: TimezoneEnum,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::action::Entity")]
    Action,
    #[sea_orm(has_many = "super::action_track::Entity")]
    ActionTrack,
    #[sea_orm(has_many = "super::ambition::Entity")]
    Ambition,
    #[sea_orm(has_many = "super::challenge::Entity")]
    Challenge,
    #[sea_orm(has_many = "super::desired_state::Entity")]
    DesiredState,
    #[sea_orm(has_many = "super::memo::Entity")]
    Memo,
    #[sea_orm(has_many = "super::reading_note::Entity")]
    ReadingNote,
    #[sea_orm(has_many = "super::tag::Entity")]
    Tag,
}

impl Related<super::action::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Action.def()
    }
}

impl Related<super::action_track::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ActionTrack.def()
    }
}

impl Related<super::ambition::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ambition.def()
    }
}

impl Related<super::challenge::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Challenge.def()
    }
}

impl Related<super::desired_state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::DesiredState.def()
    }
}

impl Related<super::memo::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Memo.def()
    }
}

impl Related<super::reading_note::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReadingNote.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
