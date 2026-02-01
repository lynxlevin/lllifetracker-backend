use crate::{
    m20250908_000001_add_type_to_tags_table::TagType as OldTagType,
    m20250928_000001_create_web_push_subscriptions_and_notification_rules_table::NotificationType as OldNotificationType,
};
use sea_orm_migration::prelude::{
    async_trait,
    extension::postgres::Type,
    sea_orm::{
        self, ActiveEnum, ConnectionTrait, DbBackend, DeriveActiveEnum, DeriveIden, EnumIter,
        Schema,
    },
    ColumnDef, DbErr, DeriveMigrationName, MigrationTrait, SchemaManager, Table,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        notification_type(manager, true).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(NotificationRule::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(NotificationRule::Type)
                            .custom(NotificationType::name())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .rename_table(
                Table::rename()
                    .table(DesiredState::Table, Direction::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(DesiredStateCategory::Table, DirectionCategory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .rename_column(Tag::DesiredStateId, Tag::DirectionId)
                    .to_owned(),
            )
            .await?;
        tag_type(manager, true).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        tag_type(manager, false).await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .rename_column(Tag::DirectionId, Tag::DesiredStateId)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(DirectionCategory::Table, DesiredStateCategory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(Direction::Table, DesiredState::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(NotificationRule::Table)
                    .drop_column(NotificationRule::Type)
                    .to_owned(),
            )
            .await?;
        notification_type(manager, false).await?;
        Ok(())
    }
}
async fn notification_type<'a>(manager: &SchemaManager<'_>, up: bool) -> Result<(), DbErr> {
    let schema = Schema::new(DbBackend::Postgres);
    if up {
        manager
            .alter_table(
                Table::alter()
                    .table(NotificationRule::Table)
                    .drop_column(NotificationRule::Type)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().name(OldNotificationType::name()).to_owned())
            .await?;
        manager
            .create_type(schema.create_enum_from_active_enum::<NotificationType>())
            .await?;
        Ok(())
    } else {
        manager
            .drop_type(Type::drop().name(NotificationType::name()).to_owned())
            .await?;
        manager
            .create_type(schema.create_enum_from_active_enum::<OldNotificationType>())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(NotificationRule::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(NotificationRule::Type)
                            .custom(OldNotificationType::name())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

async fn tag_type<'a>(manager: &SchemaManager<'_>, up: bool) -> Result<(), DbErr> {
    let schema = Schema::new(DbBackend::Postgres);
    if up {
        manager
            .alter_type(Type::alter().name("tag_type").rename_to("tag_type_old"))
            .await?;
        manager
            .create_type(schema.create_enum_from_active_enum::<TagType>())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .rename_column(Tag::Type, Tag::TypeOld)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .add_column_if_not_exists(ColumnDef::new(Tag::Type).custom(TagType::name()))
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE tag
                SET type =
                    CASE
                        WHEN type_old = 'Ambition'::tag_type_old THEN 'Ambition'::tag_type
                        WHEN type_old = 'DesiredState'::tag_type_old THEN 'Direction'::tag_type
                        WHEN type_old = 'Action'::tag_type_old THEN 'Action'::tag_type
                        ELSE 'Plain'::tag_type
                    END;
            ",
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .drop_column(Tag::TypeOld)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .modify_column(ColumnDef::new(Tag::Type).custom(TagType::name()).not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().name("tag_type_old").to_owned())
            .await?;

        Ok(())
    } else {
        manager
            .alter_type(
                Type::alter()
                    .name(TagType::name())
                    .rename_to("tag_type_new"),
            )
            .await?;
        manager
            .create_type(schema.create_enum_from_active_enum::<OldTagType>())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .rename_column(Tag::Type, "type_new")
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .add_column_if_not_exists(ColumnDef::new(Tag::Type).custom(OldTagType::name()))
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE tag
                SET type =
                    CASE
                        WHEN type_new = 'Ambition'::tag_type_new THEN 'Ambition'::tag_type
                        WHEN type_new = 'Direction'::tag_type_new THEN 'DesiredState'::tag_type
                        WHEN type_new = 'Action'::tag_type_new THEN 'Action'::tag_type
                        ELSE 'Plain'::tag_type
                    END;
            ",
        )
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .drop_column("type_new")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().name("tag_type_new").to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .modify_column(
                        ColumnDef::new(Tag::Type)
                            .custom(OldTagType::name())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum DesiredState {
    Table,
}

#[derive(DeriveIden)]
pub enum Direction {
    Table,
}

#[derive(DeriveIden)]
pub enum DesiredStateCategory {
    Table,
}

#[derive(DeriveIden)]
pub enum DirectionCategory {
    Table,
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Type,
    TypeOld,
    DesiredStateId,
    DirectionId,
}

#[derive(DeriveIden)]
pub enum NotificationRule {
    Table,
    Type,
}

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "notification_type")]
enum NotificationType {
    #[sea_orm(string_value = "Ambition")]
    Ambition,
    #[sea_orm(string_value = "Direction")]
    Direction,
    #[sea_orm(string_value = "AmbitionOrDirection")]
    AmbitionOrDirection,
    #[sea_orm(string_value = "UnaccomplishedAction")]
    UnaccomplishedAction,
}

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tag_type")]
enum TagType {
    #[sea_orm(string_value = "Ambition")]
    Ambition,
    #[sea_orm(string_value = "Direction")]
    Direction,
    #[sea_orm(string_value = "Action")]
    Action,
    #[sea_orm(string_value = "Plain")]
    Plain,
}
