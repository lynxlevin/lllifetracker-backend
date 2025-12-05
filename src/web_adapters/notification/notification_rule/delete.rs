use actix_web::{
    delete,
    web::{Data, Query, ReqData},
    HttpResponse,
};
use db_adapters::notification_rule_adapter::NotificationRuleAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::notification::notification_rule::{
    delete::delete_notification_rules, types::NotificationRuleDeleteQuery,
};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Deleting user's notification_rules.", skip(db, user))]
#[delete("")]
pub async fn delete_notification_rules_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    query: Query<NotificationRuleDeleteQuery>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match delete_notification_rules(
                user.into_inner(),
                NotificationRuleAdapter::init(&db),
                query.into_inner(),
            )
            .await
            {
                Ok(_) => HttpResponse::NoContent().finish(),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
