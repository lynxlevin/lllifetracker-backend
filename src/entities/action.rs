//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "action")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub description: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::objectives_actions::Entity")]
    ObjectivesActions,
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

impl Related<super::objectives_actions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ObjectivesActions.def()
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

impl Related<super::objective::Entity> for Entity {
    fn to() -> RelationDef {
        super::objectives_actions::Relation::Objective.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::objectives_actions::Relation::Action.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
