use sea_orm_migration::{
    prelude::{
        async_trait,
        extension::postgres::Type,
        sea_orm::{self, ActiveEnum, DeriveActiveEnum, DeriveIden, EnumIter},
        ColumnDef, DbErr, DeriveMigrationName, Expr, ForeignKey, ForeignKeyAction, Index,
        MigrationTrait, SchemaManager, Table,
    },
    schema::{big_integer_null, small_integer, string, string_len, time, uuid, uuid_null},
    sea_orm::{Condition, DbBackend, Schema},
};

const WEB_PUSH_SUBSCRIPTION_INDEX_NAME: &str = "web_push_subscription_user_id_index";
const NOTIFICATION_RULE_INDEX_NAME: &str = "notification_rule_weekday_time_index";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);
        manager
            .create_type(schema.create_enum_from_active_enum::<NotificationType>())
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(WebPushSubscription::Table)
                    .if_not_exists()
                    .col(uuid(WebPushSubscription::Id).primary_key())
                    .col(uuid(WebPushSubscription::UserId).unique_key())
                    .col(string_len(WebPushSubscription::DeviceName, 64))
                    .col(string(WebPushSubscription::Endpoint))
                    .col(big_integer_null(WebPushSubscription::ExpirationEpochTime))
                    .col(string(WebPushSubscription::P256dhKey))
                    .col(string(WebPushSubscription::AuthKey))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-web_push_subscription-user_id")
                            .from(WebPushSubscription::Table, WebPushSubscription::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(WEB_PUSH_SUBSCRIPTION_INDEX_NAME)
                    .table(WebPushSubscription::Table)
                    .col(WebPushSubscription::UserId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(NotificationRule::Table)
                    .if_not_exists()
                    .col(uuid(NotificationRule::Id).primary_key())
                    .col(uuid(NotificationRule::UserId))
                    .col(
                        ColumnDef::new(NotificationRule::Type)
                            .custom(NotificationType::name())
                            .not_null(),
                    )
                    .col(
                        small_integer(NotificationRule::Weekday)
                            .check(
                                Condition::all()
                                    .add(Expr::col(NotificationRule::Weekday).gte(0))
                                    .add(Expr::col(NotificationRule::Weekday).lte(6)),
                            )
                            .comment("Starts from Monday=0"),
                    )
                    .col(time(NotificationRule::UtcTime))
                    .col(uuid_null(NotificationRule::ActionId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-notification_rule-user_id")
                            .from(NotificationRule::Table, NotificationRule::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-notification_rule-action_id")
                            .from(NotificationRule::Table, NotificationRule::ActionId)
                            .to(Action::Table, Action::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(NOTIFICATION_RULE_INDEX_NAME)
                    .table(NotificationRule::Table)
                    .col(NotificationRule::Weekday)
                    .col(NotificationRule::UtcTime)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name(NOTIFICATION_RULE_INDEX_NAME).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NotificationRule::Table).to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name(WEB_PUSH_SUBSCRIPTION_INDEX_NAME)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(WebPushSubscription::Table).to_owned())
            .await?;
        manager
            .drop_type(
                Type::drop()
                    .if_exists()
                    .name(NotificationType::name())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum WebPushSubscription {
    Table,
    Id,
    UserId,
    DeviceName,
    Endpoint,
    ExpirationEpochTime,
    P256dhKey,
    AuthKey,
}

#[derive(DeriveIden)]
pub enum NotificationRule {
    Table,
    Id,
    UserId,
    Type,
    Weekday,
    UtcTime,
    ActionId,
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum Action {
    Table,
    Id,
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
}
