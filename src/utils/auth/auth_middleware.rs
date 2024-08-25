use std::{
    cell::RefCell,
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
use uuid::uuid;

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

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Hi from start. You requested: {}", req.path());
        let svc = self.service.clone();
        Box::pin(async move {
            match req.app_data::<Data<AppState>>() {
                Some(data) => {
                    let user: user::ActiveModel = user_service::Query::find_by_id(
                        &data.conn,
                        uuid!("22cae525-8dca-4c1f-bfb9-8efe15ef65e3"),
                    )
                    .await
                    .unwrap()
                    .unwrap()
                    .into();
                    println!("user_from_db: {}", user.id.unwrap())
                }
                None => println!("No db found"),
            };
            let session = req.get_session();
            // MYMEMO: This session is somehow empty.
            println!("{:?}", session.entries());
            let user_id = match session_user_id(&session).await {
                Ok(user_id) => {
                    println!("Hi, user_id is {}", user_id);
                    Some(user_id)
                }
                Err(e) => {
                    println!(
                        "Error getting user_id from session in the middleware! {}",
                        e
                    );
                    None
                }
            };

            let res = svc.call(req).await?;

            println!("Hi from response");
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
