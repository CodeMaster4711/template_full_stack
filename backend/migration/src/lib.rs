pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_key;
mod m20240101_000002_create_invalid_jwt;
mod m20240101_000003_create_user;
mod m20240101_000004_add_2fa_fields;
mod m20240101_000005_add_rbac_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_key::Migration),
            Box::new(m20240101_000002_create_invalid_jwt::Migration),
            Box::new(m20240101_000003_create_user::Migration),
            Box::new(m20240101_000004_add_2fa_fields::Migration),
            Box::new(m20240101_000005_add_rbac_tables::Migration),
        ]
    }
}
