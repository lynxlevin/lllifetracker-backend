use sea_orm_migration::prelude::cli;

#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
