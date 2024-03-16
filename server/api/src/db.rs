use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use tracing::info;

pub async fn connect_db() -> DatabaseConnection {
    let db_uri = std::env::var("DATABASE_URL").unwrap();

    let db = Database::connect(db_uri).await;
    info!("Database Ready: {}", db.is_ok());

    db.unwrap()
}

pub async fn run_migration(db: &DatabaseConnection) {
    info!("Running Migrations");
    Migrator::up(db, None).await.unwrap();
}
