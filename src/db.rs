use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::{Connection, MysqlConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use std::env;
use tracing::warn;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type Database = Pool<ConnectionManager<MysqlConnection>>;

#[tracing::instrument]
pub fn get_db_connection() -> Database {
    let span = tracing::info_span!("Connecting to the database");
    let _span_guard = span.enter();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let mut conn;
    loop {
        match MysqlConnection::establish(&database_url) {
            Ok(x) => {
                conn = x;
                break;
            }
            Err(_) => warn!("Error connecting to {}", database_url),
        }
    }

    conn.run_pending_migrations(MIGRATIONS)
        .expect("Migrate fail");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    diesel::r2d2::Pool::builder()
        .max_size(4)
        .test_on_check_out(true)
        .build(manager)
        .expect("Cant create db pool")
}
