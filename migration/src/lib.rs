pub use sea_orm_migration::prelude::{async_trait, MigrationTrait, MigratorTrait};

mod m20240722_000001_create_users_table;
mod m20240927_000001_create_ambitions_table;
mod m20240927_000002_create_objectives_table;
mod m20240927_000003_create_actions_table;
mod m20240927_000004_create_ambitions_objectives_table;
mod m20240927_000005_create_objectives_actions_table;
mod m20240927_000006_create_tags_table;
mod m20240927_000007_create_records_table;
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
mod m20250313_000001_rename_mission_memos_to_challenges_table;
mod m20250313_000002_rename_book_excerpts_to_reading_notes_table;
mod m20250315_000001_add_ordering_to_ambitions_and_desired_states_table;
mod m20250315_000002_create_diaries_table;
mod m20250316_000001_create_diaries_tags_table;
mod m20250322_000001_add_color_to_actions_table;
mod m20250407_000001_add_unique_constraint_to_action_tracks_table;
mod m20250410_000001_add_track_type_to_actions_table;
mod m20250411_000001_modify_action_id_to_required_in_action_tracks_table;
mod m20250412_000001_drop_my_way_link_tables;
mod m20250412_000002_drop_memos_and_challenges_tables;
mod m20250415_000001_add_name_to_tags_table;
mod m20250430_000001_create_mindset_table;
mod m20250610_000001_create_desired_state_categories_table;
mod m20250614_000001_remove_mindset_table;
mod m20250626_000001_add_focus_to_desired_state_table;
mod m20250708_000001_add_first_track_at_to_users_table;
mod m20250712_000001_remove_score_from_diaries_table;
mod m20250808_000001_create_action_goals_table;
mod m20250813_000001_unique_constraint_for_action_goals_table;
mod m20250814_000001_create_thinking_notes_with_tags_table;
mod m20250908_000001_add_type_to_tags_table;
mod m_seed_data;

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
            Box::new(m20250313_000001_rename_mission_memos_to_challenges_table::Migration),
            Box::new(m20250313_000002_rename_book_excerpts_to_reading_notes_table::Migration),
            Box::new(
                m20250315_000001_add_ordering_to_ambitions_and_desired_states_table::Migration,
            ),
            Box::new(m20250315_000002_create_diaries_table::Migration),
            Box::new(m20250316_000001_create_diaries_tags_table::Migration),
            Box::new(m20250322_000001_add_color_to_actions_table::Migration),
            Box::new(m20250407_000001_add_unique_constraint_to_action_tracks_table::Migration),
            Box::new(m20250410_000001_add_track_type_to_actions_table::Migration),
            Box::new(
                m20250411_000001_modify_action_id_to_required_in_action_tracks_table::Migration,
            ),
            Box::new(m20250412_000001_drop_my_way_link_tables::Migration),
            Box::new(m20250412_000002_drop_memos_and_challenges_tables::Migration),
            Box::new(m20250415_000001_add_name_to_tags_table::Migration),
            Box::new(m20250430_000001_create_mindset_table::Migration),
            Box::new(m20250610_000001_create_desired_state_categories_table::Migration),
            Box::new(m20250614_000001_remove_mindset_table::Migration),
            Box::new(m20250626_000001_add_focus_to_desired_state_table::Migration),
            Box::new(m20250708_000001_add_first_track_at_to_users_table::Migration),
            Box::new(m20250712_000001_remove_score_from_diaries_table::Migration),
            Box::new(m20250808_000001_create_action_goals_table::Migration),
            Box::new(m20250813_000001_unique_constraint_for_action_goals_table::Migration),
            Box::new(m20250814_000001_create_thinking_notes_with_tags_table::Migration),
            Box::new(m20250908_000001_add_type_to_tags_table::Migration),
        ]
    }
}
