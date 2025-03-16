use entities::{
    action, action_track, ambition, ambitions_desired_states, challenge, challenges_tags,
    desired_state, desired_states_actions, diary, diaries_tags, memo, memos_tags, reading_note,
    reading_notes_tags, tag, user,
};

use sea_orm::{
    sea_query::TableCreateStatement, ConnectionTrait, Database, DbBackend, DbConn, DbErr, Schema,
};

pub mod factory;

pub use factory::{
    ActionFactory, ActionTrackFactory, AmbitionFactory, ChallengeFactory, DesiredStateFactory,
    MemoFactory, ReadingNoteFactory, UserFactory,
};

pub async fn init_db() -> Result<DbConn, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;
    let schema = Schema::new(DbBackend::Sqlite);
    let mut stmts: Vec<TableCreateStatement> = vec![];
    stmts.push(schema.create_table_from_entity(user::Entity));
    stmts.push(schema.create_table_from_entity(ambition::Entity));
    stmts.push(schema.create_table_from_entity(desired_state::Entity));
    stmts.push(schema.create_table_from_entity(action::Entity));
    stmts.push(schema.create_table_from_entity(ambitions_desired_states::Entity));
    stmts.push(schema.create_table_from_entity(desired_states_actions::Entity));
    stmts.push(schema.create_table_from_entity(tag::Entity));
    stmts.push(schema.create_table_from_entity(memo::Entity));
    stmts.push(schema.create_table_from_entity(memos_tags::Entity));
    stmts.push(schema.create_table_from_entity(challenge::Entity));
    stmts.push(schema.create_table_from_entity(challenges_tags::Entity));
    stmts.push(schema.create_table_from_entity(reading_note::Entity));
    stmts.push(schema.create_table_from_entity(reading_notes_tags::Entity));
    stmts.push(schema.create_table_from_entity(action_track::Entity));
    stmts.push(schema.create_table_from_entity(diary::Entity));
    stmts.push(schema.create_table_from_entity(diaries_tags::Entity));

    for stmt in stmts {
        let _ = &db.execute(db.get_database_backend().build(&stmt)).await?;
    }
    Ok(db)
}
