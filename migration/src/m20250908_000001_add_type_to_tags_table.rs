use sea_orm_migration::{
    prelude::{
        async_trait,
        extension::postgres::Type,
        sea_orm::{self, DeriveIden},
        ColumnDef, ConnectionTrait, DbErr, DeriveMigrationName, MigrationTrait, SchemaManager,
        Table,
    },
    sea_orm::{ActiveEnum, DbBackend, DeriveActiveEnum, EnumIter, Schema},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<TagType>())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Tag::Type)
                            .custom(TagType::name())
                            .not_null()
                            .default(TagType::Plain),
                    )
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE tag
                SET type =
                    CASE
                        WHEN ambition_id is not NULL THEN 'Ambition'::tag_type
                        WHEN desired_state_id is not NULL THEN 'DesiredState'::tag_type
                        WHEN action_id is not NULL THEN 'Action'::tag_type
                        ELSE 'Plain'::tag_type
                    END;
            ",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE tag
                ADD CONSTRAINT type_foreign_keys_compatibility
                CHECK (
                    (type = 'Ambition' AND ambition_id is not null AND desired_state_id is null AND action_id is null AND name is null) OR
                    (type = 'DesiredState' AND desired_state_id is not null AND ambition_id is null AND action_id is null AND name is null) OR
                    (type = 'Action' AND action_id is not null AND ambition_id is null AND desired_state_id is null AND name is null) OR
                    (type = 'Plain' AND name is not null AND ambition_id is null AND desired_state_id is null AND action_id is null)
                );
            ",
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared("ALTER TABLE tag DROP CONSTRAINT type_foreign_keys_compatibility;")
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Tag::Table)
                    .drop_column(Tag::Type)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(TagType::name()).to_owned())
            .await?;
        Ok(())
    }
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
