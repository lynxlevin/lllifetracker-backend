use crate::entities::{
    action, ambition, ambitions_objectives, objective, objectives_actions, record, tag, user,
};
use sea_orm::{
    sea_query::TableCreateStatement, ConnectionTrait, Database, DbBackend, DbConn, DbErr, Schema,
};

pub mod seed;

#[cfg(test)]
pub async fn init_db() -> Result<DbConn, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let schema = Schema::new(DbBackend::Sqlite);
    let mut stmts: Vec<TableCreateStatement> = vec![];
    stmts.push(schema.create_table_from_entity(user::Entity));
    stmts.push(schema.create_table_from_entity(ambition::Entity));
    stmts.push(schema.create_table_from_entity(objective::Entity));
    stmts.push(schema.create_table_from_entity(action::Entity));
    stmts.push(schema.create_table_from_entity(ambitions_objectives::Entity));
    stmts.push(schema.create_table_from_entity(objectives_actions::Entity));
    stmts.push(schema.create_table_from_entity(tag::Entity));
    stmts.push(schema.create_table_from_entity(record::Entity));

    for stmt in stmts {
        let _ = &db.execute(db.get_database_backend().build(&stmt)).await?;
    }
    Ok(db)
}
