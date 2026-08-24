use common::{db::init_test_db, settings::get_test_settings};
pub use entities; // This is for db.get_schema_registry to find entities.

#[actix_web::main]
async fn main() -> () {
    let settings = get_test_settings();
    init_test_db(&settings).await;
    ()
}
