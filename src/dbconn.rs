pub mod dbconnection {
    use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
    pub async fn connection_pool() -> SqlitePool{
        let connpool = SqlitePoolOptions::new().max_connections(10).connect_lazy("sqlite://datesapp").unwrap();
        connpool
    }
}