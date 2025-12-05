use actix_web::{
    post,
    web::{Data, Json, ReqData},
    HttpResponse,
};
use db_adapters::notification_rule_adapter::NotificationRuleAdapter;
use entities::user as user_entity;
use sea_orm::DbConn;
use use_cases::notification::notification_rule::{
    create::create_notification_rules, types::NotificationRuleCreateRequest,
};

use crate::utils::{response_401, response_500};

#[tracing::instrument(name = "Creating user's notification_rules.", skip(db, user))]
#[post("")]
pub async fn create_notification_rules_endpoint(
    db: Data<DbConn>,
    user: Option<ReqData<user_entity::Model>>,
    req: Json<NotificationRuleCreateRequest>,
) -> HttpResponse {
    match user {
        Some(user) => {
            match create_notification_rules(
                user.into_inner(),
                NotificationRuleAdapter::init(&db),
                req.into_inner(),
            )
            .await
            {
                Ok(res) => HttpResponse::Created().json(res),
                Err(e) => response_500(e),
            }
        }
        None => response_401(),
    }
}
