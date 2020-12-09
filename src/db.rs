use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::{Connection, MysqlConnection};
use std::env;

embed_migrations!();

pub type Database = Pool<ConnectionManager<MysqlConnection>>;

pub fn get_db_connection() -> Database {
    let span = tracing::info_span!("Connecting to the database");
    let _span_guard = span.enter();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    embedded_migrations::run_with_output(&conn, &mut std::io::stdout())
        .expect("Unable to run migrations");

    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    diesel::r2d2::Pool::builder()
        .max_size(4)
        .test_on_check_out(true)
        .build(manager)
        .expect("Cant create db pool")
}
