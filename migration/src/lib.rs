pub use sea_orm_migration::prelude::*;

mod m20240722_000001_create_users_table;
mod m20240927_000001_create_ambitions_table;
mod m20240927_000002_create_objectives_table;
mod m20240927_000003_create_actions_table;
mod m20240927_000004_create_ambitions_objectives_table;
mod m20240927_000005_create_objectives_actions_table;
mod m20240927_000006_create_tags_table;
mod m20240927_000007_create_records_table;
mod m_seed_data;
mod m20241124_000001_create_memos_table;
mod m20241124_000002_create_memos_tags_table;
mod m20241201_000001_add_description_to_objective_action_tables;
mod m20241218_000001_create_mission_memos_table;
mod m20241218_000002_create_mission_memos_tags_table;
mod m20241222_000001_create_book_excerpts_table;
mod m20241222_000002_create_book_excerpts_tags_table;
mod m20241226_000001_add_archived_to_ambition_objective_action_tables;
mod m20241227_000001_remove_records_table;
mod m20241227_000002_create_action_tracks_table;
mod m20250219_000001_add_ordering_trackable_to_actions_table;
mod m20250303_000001_add_favorite_to_memos_table;
mod m20250308_000001_rename_objectives_to_desired_states_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240722_000001_create_users_table::Migration),
            Box::new(m20240927_000001_create_ambitions_table::Migration),
            Box::new(m20240927_000002_create_objectives_table::Migration),
            Box::new(m20240927_000003_create_actions_table::Migration),
            Box::new(m20240927_000004_create_ambitions_objectives_table::Migration),
            Box::new(m20240927_000005_create_objectives_actions_table::Migration),
            Box::new(m20240927_000006_create_tags_table::Migration),
            Box::new(m20240927_000007_create_records_table::Migration),
            Box::new(m_seed_data::Migration),
            Box::new(m20241124_000001_create_memos_table::Migration),
            Box::new(m20241124_000002_create_memos_tags_table::Migration),
            Box::new(m20241201_000001_add_description_to_objective_action_tables::Migration),
            Box::new(m20241218_000001_create_mission_memos_table::Migration),
            Box::new(m20241218_000002_create_mission_memos_tags_table::Migration),
            Box::new(m20241222_000001_create_book_excerpts_table::Migration),
            Box::new(m20241222_000002_create_book_excerpts_tags_table::Migration),
            Box::new(m20241226_000001_add_archived_to_ambition_objective_action_tables::Migration),
            Box::new(m20241227_000001_remove_records_table::Migration),
            Box::new(m20241227_000002_create_action_tracks_table::Migration),
            Box::new(m20250219_000001_add_ordering_trackable_to_actions_table::Migration),
            Box::new(m20250303_000001_add_favorite_to_memos_table::Migration),
            Box::new(m20250308_000001_rename_objectives_to_desired_states_table::Migration),
        ]
    }
}
