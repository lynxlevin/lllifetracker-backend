use db_adapters::notification_rule_adapter::{
    NotificationRuleAdapter, NotificationRuleFilter, NotificationRuleMutation,
    NotificationRuleQuery,
};
use entities::user as user_entity;

use crate::{notification::notification_rule::types::NotificationRuleDeleteQuery, UseCaseError};

pub async fn delete_notification_rules<'a>(
    user: user_entity::Model,
    notification_rule_adapter: NotificationRuleAdapter<'a>,
    query: NotificationRuleDeleteQuery,
) -> Result<(), UseCaseError> {
    let rules = notification_rule_adapter
        .clone()
        .filter_eq_user(&user)
        .filter_eq_type(query.r#type)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    notification_rule_adapter
        .delete_many(rules)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
