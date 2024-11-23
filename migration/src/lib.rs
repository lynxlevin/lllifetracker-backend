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
        ]
    }
}
