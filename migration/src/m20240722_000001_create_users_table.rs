use sea_orm_migration::{
    prelude::{
        async_trait,
        extension::postgres::Type,
        sea_orm::{self, DeriveIden, EnumIter, Iden, Iterable},
        Alias, DbErr, DeriveMigrationName, Expr, Index, MigrationTrait, SchemaManager, Table,
    },
    schema::{boolean, enumeration, string, string_uniq, timestamp_with_time_zone, uuid},
};

const INDEX_NAME: &str = "users_id_email_is_active_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(TimezoneEnum)
                    .values(TimezoneVariants::iter())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(uuid(User::Id).primary_key())
                    .col(string_uniq(User::Email))
                    .col(string(User::Password))
                    .col(string(User::FirstName))
                    .col(string(User::LastName))
                    .col(
                        enumeration(
                            User::Timezone,
                            Alias::new(TimezoneEnum.to_string()),
                            TimezoneVariants::iter(),
                        )
                        .default(TimezoneVariants::AsiaTokyo.to_string()),
                    )
                    .col(boolean(User::IsActive).default(false))
                    .col(
                        timestamp_with_time_zone(User::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(User::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(INDEX_NAME)
                    .table(User::Table)
                    .col(User::Id)
                    .col(User::Email)
                    .col(User::IsActive)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().if_exists().name(TimezoneEnum).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Email,
    Password,
    FirstName,
    LastName,
    Timezone,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
struct TimezoneEnum;

#[derive(DeriveIden, EnumIter)]
enum TimezoneVariants {
    #[sea_orm(iden = "Asia/Tokyo")]
    AsiaTokyo,
    #[sea_orm(iden = "UTC")]
    Utc,
}
