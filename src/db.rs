use sqlx::postgres::PgPool;
use sqlx::Error;

pub async fn init_db() -> Result<PgPool, Error> {
    PgPool::connect("postgres://deepwater@localhost/serverdb:5433").await
    //sqlx::query()
}
