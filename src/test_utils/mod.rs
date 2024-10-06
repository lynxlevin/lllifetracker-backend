use crate::entities::user;
use sea_orm::{DbConn, DbErr};

#[cfg(test)]
pub async fn get_or_create_user(db: &DbConn) -> Result<user::Model, DbErr> {
    use sea_orm::{prelude::*, Set};

    match user::Entity::find()
        .filter(user::Column::Email.eq("test@test.com".to_string()))
        .one(db)
        .await?
    {
        Some(user) => Ok(user),
        None => Ok(user::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            email: Set(format!("{}@test.com", uuid::Uuid::new_v4().to_string())),
            password: Set("password".to_string()),
            first_name: Set("Lynx".to_string()),
            last_name: Set("Levin".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?),
    }
}

#[cfg(test)]
pub async fn init_db() -> Result<DbConn, DbErr> {
    use crate::startup::get_database_connection;
    use migration::{Migrator, MigratorTrait};

    dotenvy::from_filename(".env.test").unwrap();
    // NOTE: Ideally, Sqlite should be used instead of Postgres but cannot,
    // because programmatic migration for Sqlite using Migrator is not supported
    // nor migration from entities do not set default constraints.
    let db = get_database_connection().await;
    Migrator::up(&db, None).await.unwrap();
    Ok(db)
}

#[cfg(test)]
pub async fn flush_actions(db: &DbConn) -> Result<(), DbErr> {
    use sea_orm::EntityTrait;

    use crate::entities::action;

    action::Entity::delete_many().exec(db).await?;
    Ok(())
}
