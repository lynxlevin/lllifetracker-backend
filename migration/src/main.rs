use sea_orm_migration::prelude::cli;

#[async_std::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
