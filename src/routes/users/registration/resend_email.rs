use crate::services::user::Query as UserQuery;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    email: String,
}
#[tracing::instrument(name = "Resending registration confirmation email", skip(data, req))]
#[post("/resend-email")]
pub async fn resend_email(
    data: Data<crate::startup::AppState>,
    req: Json<RequestBody>,
) -> HttpResponse {
    let user = match UserQuery::find_inactive_by_email(&data.conn, req.email.clone()).await {
        Ok(user) => user.unwrap(),
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "User not found : {:#?}", e);
            return HttpResponse::InternalServerError().json(crate::types::ErrorResponse {
                error: "User with this email was not found. This error happens if you have already activated this user.".to_string(),
            });
        }
    };

    let redis_con = &mut data
        .redis_pool
        .get()
        .await
        .map_err(|e| {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            HttpResponse::InternalServerError().json(crate::types::ErrorResponse {
                error: "We cannot activate your account at the moment".to_string(),
            })
        })
        .expect("Redis connection cannot be gotten.");

    crate::utils::emails::send_multipart_email(
        "Let's get you verified".to_string(),
        user.id,
        user.email,
        user.first_name,
        user.last_name,
        "verification_email.html",
        redis_con,
    )
    .await
    .unwrap();

    tracing::event!(target: "backend", tracing::Level::INFO, "Verification email re-sent successfully.");

    HttpResponse::Ok().json(crate::types::SuccessResponse { message: "Account activation link has been sent to your email address. Kindly take action before its expiration".to_string() })
}
