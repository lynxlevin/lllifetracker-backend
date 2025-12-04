use actix_web::{
    get,
    web::{Data, ReqData},
    HttpResponse,
};
use db_adapters::notification_rule_adapter::NotificationRuleAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::notification::notification_rule::list::list_notification_rules;

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Listing user's notification_rules.", skip(db, user))]
#[get("")]
pub async fn list_notification_rules_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match list_notification_rules(user.into_inner(), NotificationRuleAdapter::init(&db))
                .await
            {
                Ok(res) => HttpResponse::Ok().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
