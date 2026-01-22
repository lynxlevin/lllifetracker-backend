use sea_orm_migration::prelude::{
    async_trait,
    extension::postgres::Type,
    sea_orm::{self, ActiveEnum, DeriveActiveEnum, EnumIter},
    DbErr, DeriveMigrationName, MigrationTrait, SchemaManager,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_type(
                Type::alter()
                    .name(NotificationType::name())
                    .add_value("FocusedDesiredState")
                    .if_not_exists(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // NOTE: value addition to Postgres custom type is irreversible.
        Ok(())
    }
}

#[derive(DeriveActiveEnum, EnumIter)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "notification_type")]
enum NotificationType {
    #[sea_orm(string_value = "Ambition")]
    Ambition,
    #[sea_orm(string_value = "DesiredState")]
    DesiredState,
    #[sea_orm(string_value = "Action")]
    Action,
    #[sea_orm(string_value = "AmbitionOrDesiredState")]
    AmbitionOrDesiredState,
    #[sea_orm(string_value = "FocusedDesiredState")]
    FocusedDesiredState,
}
