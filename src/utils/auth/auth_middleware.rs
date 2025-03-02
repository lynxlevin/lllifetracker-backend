use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::auth::session::get_user_id;
use actix_session::SessionExt;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::Data,
    Error, HttpMessage,
};
use futures::future::LocalBoxFuture;
use sea_orm::DbConn;
use services::user::Query as UserQuery;

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
            match set_user(&req).await {
                Ok(_) => (),
                Err(e) => {
                    // MYMEMO: use log
                    println!("Error in the auth middleware! {e}");
                }
            }

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

async fn set_user(req: &ServiceRequest) -> Result<(), String> {
    let session = req.get_session();
    let user_id = match get_user_id(&session).await {
        Ok(id) => id,
        Err(e) => {
            return Err(e);
        }
    };

    let user = match req.app_data::<Data<DbConn>>() {
        Some(data) => match UserQuery::find_by_id(data, user_id).await {
            Ok(user) => match user {
                Some(user) => user,
                None => {
                    return Err("No user found for the user_id".to_string());
                }
            },
            Err(e) => {
                return Err(e.to_string());
            }
        },
        None => {
            return Err("Error acquiring DB connection.".to_string());
        }
    };

    req.extensions_mut().insert(user);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_session::SessionExt;
    use actix_web::test;
    use sea_orm::prelude::*;

    use ::types::{USER_EMAIL_KEY, USER_ID_KEY};
    use entities::user;
    use test_utils::{self, *};

    #[actix_web::test]
    async fn test_set_user() -> Result<(), String> {
        let db = test_utils::init_db().await.unwrap();
        let user = factory::user().insert(&db).await.unwrap();
        let srv_req = test::TestRequest::default()
            .app_data(Data::new(db.clone()))
            .to_srv_request();
        srv_req.get_session().insert(USER_ID_KEY, user.id).unwrap();
        srv_req
            .get_session()
            .insert(USER_EMAIL_KEY, user.email.clone())
            .unwrap();
        set_user(&srv_req).await?;

        let user2 = srv_req
            .extensions()
            .get::<user::Model>()
            .unwrap()
            .to_owned();
        assert_eq!(user2, user);

        Ok(())
    }
}
