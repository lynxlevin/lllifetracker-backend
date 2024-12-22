//! `SeaORM` Entity, @generated by sea-orm-codegen 1.0.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "book_excerpt")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub page_number: i16,
    #[sea_orm(column_type = "Text")]
    pub text: String,
    pub date: Date,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::book_excerpts_tags::Entity")]
    BookExcerptsTags,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    User,
}

impl Related<super::book_excerpts_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BookExcerptsTags.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        super::book_excerpts_tags::Relation::Tag.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::book_excerpts_tags::Relation::BookExcerpt.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
