use crate::infra::environment::Environment;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};

pub struct Database {
    pub host: String,
    pub name: String,
    pub user: String,
    pub pass: String,
    pub app_name: String,
    pub port: u16,
    pub min_pool_size: u32,
    pub max_pool_size: u32,
}

impl Database {
    pub fn from_env() -> Self {
        let host = Environment::string("DB_HOST", "localhost");
        let name = Environment::string("DB_NAME", "local");
        let user = Environment::string("DB_USER", "local");
        let pass = Environment::string("DB_PASS", "local");
        let port = Environment::u16("DB_PORT", 5432);
        let app_name = Environment::string("DB_APP_NAME", "outbox-pattern-processor");
        let min_pool_size = Environment::u32("DB_MIN_POOL_SIZE", 1);
        let max_pool_size = Environment::u32("DB_MAX_POOL_SIZE", 10);

        Self {
            host,
            name,
            user,
            pass,
            app_name,
            port,
            min_pool_size,
            max_pool_size,
        }
    }

    pub fn db_connection_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .database(&self.name)
            .username(&self.user)
            .password(&self.pass)
            .port(self.port)
            .application_name(&self.app_name)
    }

    pub async fn create_db_pool(self) -> Result<Pool<Postgres>, sqlx::Error> {
        PgPoolOptions::new()
            .min_connections(self.min_pool_size)
            .max_connections(self.max_pool_size)
            .test_before_acquire(true)
            .connect_with(self.db_connection_options())
            .await
    }
}
