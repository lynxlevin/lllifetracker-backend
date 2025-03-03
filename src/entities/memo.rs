//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "memo")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub text: String,
    pub date: Date,
    pub archived: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub favorite: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::memos_tags::Entity")]
    MemosTags,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::memos_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemosTags.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::memos_tags::Relation::Tag.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::memos_tags::Relation::Memo.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
