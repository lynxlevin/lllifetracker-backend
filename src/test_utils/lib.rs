use common::{db::init_db as init_db_fn, settings::get_test_settings};
use sea_orm::{DbConn, DbErr};

pub async fn init_db() -> Result<DbConn, DbErr> {
    let settings = get_test_settings();
    let db_conn = init_db_fn(&settings).await;
    Ok(db_conn)
}
