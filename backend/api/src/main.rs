use sea_orm::DbConn;
use sea_orm_migration::MigratorTrait;
use std::net::SocketAddr;

mod auth;
mod auth_service;
mod db;
mod init;
mod rbac_service;
mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db_conn: DbConn,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("api=info".parse().unwrap()),
        )
        .init();

    let db_conn = match db::establish_connection().await {
        Ok(conn) => {
            tracing::info!("database connection established");
            conn
        }
        Err(e) => {
            tracing::error!(error = %e, "failed to connect to database");
            std::process::exit(1);
        }
    };

    // Run migrations automatically on startup
    if let Err(e) = migration::Migrator::up(&db_conn, None).await {
        tracing::error!(error = %e, "failed to run migrations");
        std::process::exit(1);
    }
    tracing::info!("migrations applied");

    if let Err(e) = init::initialize_database(&db_conn).await {
        tracing::error!(error = %e, "failed to initialize database");
        std::process::exit(1);
    }

    let state = AppState {
        db_conn: db_conn.clone(),
    };

    let app = routes::create_router().with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!(addr = %addr, "server listening");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
