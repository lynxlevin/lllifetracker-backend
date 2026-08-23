use sea_orm_migration::{
    prelude::{
        async_trait,
        extension::postgres::Type,
        sea_orm::{self, DeriveIden, EnumIter},
        DbErr, DeriveMigrationName, MigrationTrait, SchemaManager, Table,
    },
    schema::string,
    sea_orm::{ActiveEnum, DeriveActiveEnum},
    sea_query::Index,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .modify_column(string(User::Timezone).default("Asia/Tokyo"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(TimezoneEnum).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Action::Table)
                    .modify_column(string(Action::TrackType).default("TimeSpan"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(ActionTrackType::name()).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .modify_column(string(Tag::Type).default("Plain"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(TagType::name()).to_owned())
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(NotificationRule::Table)
                    .modify_column(string(NotificationRule::Type))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(NotificationType::name()).to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("action_goal_to_date_unique_index").to_owned())
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Timezone,
}
#[derive(DeriveIden)]
struct TimezoneEnum;

#[derive(DeriveIden)]
pub enum Action {
    Table,
    TrackType,
}
#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "action_track_type")]
enum ActionTrackType {
    #[sea_orm(string_value = "TimeSpan")]
    TimeSpan,
    #[sea_orm(string_value = "Count")]
    Count,
}

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Type,
}
#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "tag_type")]
pub enum TagType {
    #[sea_orm(string_value = "Ambition")]
    Ambition,
    #[sea_orm(string_value = "DesiredState")]
    DesiredState,
    #[sea_orm(string_value = "Action")]
    Action,
    #[sea_orm(string_value = "Plain")]
    Plain,
}

#[derive(DeriveIden)]
pub enum NotificationRule {
    Table,
    Type,
}
#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "notification_type")]
pub enum NotificationType {
    #[sea_orm(string_value = "Ambition")]
    Ambition,
    #[sea_orm(string_value = "Direction")]
    Direction,
    #[sea_orm(string_value = "AmbitionOrDirection")]
    AmbitionOrDirection,
    #[sea_orm(string_value = "UnaccomplishedAction")]
    UnaccomplishedAction,
}
