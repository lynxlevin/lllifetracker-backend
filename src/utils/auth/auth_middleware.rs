use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_session::SessionExt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpMessage,
};
use futures::future::LocalBoxFuture;

use crate::{entities::user, services::user as user_service, startup::AppState};

pub struct AuthenticateUser;

impl<S: 'static, B> Transform<S, ServiceRequest> for AuthenticateUser
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticateUserMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticateUserMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticateUserMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticateUserMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    // MYMEMO: maybe add redirect if not logged in.
    // MYMEMO: then, maybe there's a way to make user not-optional?
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            let session = req.get_session();
            // MYMEMO: refactor
            match session_user_id(&session).await {
                Ok(user_id) => match req.app_data::<Data<AppState>>() {
                    Some(data) => {
                        match user_service::Query::find_by_id(&data.conn, user_id)
                            .await
                            .unwrap()
                        {
                            Some(user) => {
                                let user: user::ActiveModel = user.into();
                                req.extensions_mut().insert(user);
                            }
                            None => (),
                        }
                    }
                    None => (),
                },
                Err(e) => {
                    println!(
                        "Error getting user_id from session in the middleware! {}",
                        e
                    );
                }
            };

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

// MYMEMO: Make this a common function
async fn session_user_id(session: &actix_session::Session) -> Result<uuid::Uuid, String> {
    match session.get(crate::types::USER_ID_KEY) {
        Ok(user_id) => match user_id {
            None => Err("You are not authenticated".to_string()),
            Some(id) => Ok(id),
        },
        Err(e) => Err(format!("{e}")),
    }
}
