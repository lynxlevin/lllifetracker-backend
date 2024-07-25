use sea_orm::sea_query;
use sea_orm_migration::prelude::*;
use uuid::Uuid;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(User::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(User::Email).text().unique_key().not_null())
                    .col(ColumnDef::new(User::Password).text().not_null())
                    .col(ColumnDef::new(User::FirstName).text().not_null())
                    .col(ColumnDef::new(User::LastName).text().not_null())
                    .col(ColumnDef::new(User::IsActive).boolean().default(false))
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(chrono::prelude::Utc::now()),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(chrono::prelude::Utc::now()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                sea_query::Index::create()
                    .name("users_id_email_is_active_index")
                    .table(User::Table)
                    .col(User::Id)
                    .col(User::Email)
                    .col(User::IsActive)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    Password,
    FirstName,
    LastName,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
