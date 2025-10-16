use common::settings::types::Settings;
use db_adapters::{
    ambition_adapter::{AmbitionAdapter, AmbitionFilter, AmbitionQuery},
    desired_state_adapter::{DesiredStateAdapter, DesiredStateFilter, DesiredStateQuery},
    web_push_subscription_adapter::{WebPushSubscriptionAdapter, WebPushSubscriptionQuery},
};
use entities::user as user_entity;
use jwt_simple::reexports::rand::{seq::SliceRandom, thread_rng};

use crate::{
    error_500, notification::web_push_request_builder::WebPushRequestBuilder, UseCaseError,
};

enum NotificationChoice {
    Ambition,
    DesiredState,
}

pub async fn send_web_push<'a>(
    user: user_entity::Model,
    settings: &Settings,
    web_push_subscription_adapter: WebPushSubscriptionAdapter<'a>,
    ambition_adapter: AmbitionAdapter<'a>,
    desired_state_adapter: DesiredStateAdapter<'a>,
) -> Result<(), UseCaseError> {
    let choices = [
        NotificationChoice::Ambition,
        NotificationChoice::DesiredState,
    ];
    let choice = choices.choose(&mut thread_rng()).unwrap();
    let message = match choice {
        NotificationChoice::Ambition => {
            let ambition = ambition_adapter
                .filter_eq_user(&user)
                .filter_eq_archived(false)
                .get_random()
                .await
                .map_err(error_500)?
                .ok_or(UseCaseError::NotFound(
                    "You don't have any ambition.".to_string(),
                ))?;
            match ambition.description {
                Some(description) => format!("{}:\n{}", ambition.name, description),
                None => ambition.name,
            }
        }
        NotificationChoice::DesiredState => {
            let desired_state = desired_state_adapter
                .filter_eq_user(&user)
                .filter_eq_archived(false)
                .get_random()
                .await
                .map_err(error_500)?
                .ok_or(UseCaseError::NotFound(
                    "You don't have any desired_state.".to_string(),
                ))?;
            match desired_state.description {
                Some(description) => format!("{}:\n{}", desired_state.name, description),
                None => desired_state.name,
            }
        }
    };

    let subscription = web_push_subscription_adapter
        .get_by_user(&user)
        .await
        .map_err(error_500)?
        .ok_or(UseCaseError::NotFound(
            "You don't have active web push subscription.".to_string(),
        ))?;

    let builder = WebPushRequestBuilder::new(&subscription, &settings)?;
    let encrypted_message = builder.encrypt_message(message)?;
    let request = builder.get_awc_client(None)?;
    request
        .send_body(encrypted_message)
        .await
        .map_err(error_500)?;

    Ok(())
}
