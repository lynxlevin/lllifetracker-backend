use sea_orm_migration::{
    prelude::{extension::postgres::Type, *},
    sea_orm::{ActiveEnum, DbBackend, DeriveActiveEnum, EnumIter, Schema},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<ActionTrackType>())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Action::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Action::TrackType)
                            .custom(ActionTrackType::name())
                            .not_null()
                            .default(ActionTrackType::TimeSpan),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Action::Table)
                    .drop_column(Action::TrackType)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .name(ActionTrackType::name())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

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
