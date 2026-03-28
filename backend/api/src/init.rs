use entity::{
    key, organization, permission, role, role_permission, user, user_organization, Key, Organization,
    Permission, Role, RolePermission, User, UserOrganization,
};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::auth::crypto::{generate_salt, hash_password, RsaKeyPair};

pub async fn initialize_database(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Initializing database with default data...");

    // 1. Create RSA key pair if not exists
    let key_exists = Key::find()
        .filter(key::Column::Name.eq("main"))
        .one(db)
        .await?
        .is_some();

    if !key_exists {
        tracing::info!("Creating RSA key pair...");
        let key_pair = RsaKeyPair::generate()?;
        let key_id = Uuid::new_v4();
        let new_key = key::ActiveModel {
            id: ActiveValue::Set(key_id),
            name: ActiveValue::Set("main".to_string()),
            private_key: ActiveValue::Set(key_pair.private_key),
        };
        Key::insert(new_key).exec_without_returning(db).await?;
        tracing::info!("RSA key pair created");
    }

    // 2. Create default organization
    let default_org_id = if let Some(org) = Organization::find()
        .filter(organization::Column::Name.eq("Default Organization"))
        .one(db)
        .await?
    {
        org.id
    } else {
        tracing::info!("Creating default organization...");
        let org_id = Uuid::new_v4();
        let now = chrono::Utc::now().naive_utc();
        let new_org = organization::ActiveModel {
            id: ActiveValue::Set(org_id),
            name: ActiveValue::Set("Default Organization".to_string()),
            description: ActiveValue::Set(Some("Default organization".to_string())),
            created_at: ActiveValue::Set(now),
            updated_at: ActiveValue::Set(now),
        };
        Organization::insert(new_org)
            .exec_without_returning(db)
            .await?;
        org_id
    };

    // 3. Create default permissions
    let permissions_to_create = vec![
        ("organization.view", "organization", "view", "View organization details"),
        ("organization.update", "organization", "update", "Update organization"),
        ("organization.delete", "organization", "delete", "Delete organization"),
        ("members.view", "members", "view", "View organization members"),
        ("members.create", "members", "create", "Add members to organization"),
        ("members.update", "members", "update", "Update member roles"),
        ("members.delete", "members", "delete", "Remove members from organization"),
        ("users.view", "users", "view", "View user list and details"),
        ("users.create", "users", "create", "Create new users"),
        ("users.update", "users", "update", "Update user details"),
        ("users.delete", "users", "delete", "Delete users"),
        ("system.manage", "system", "manage", "Manage system settings"),
    ];

    let mut permission_map = std::collections::HashMap::new();

    for (name, resource, action, description) in permissions_to_create {
        let perm_id = if let Some(perm) = Permission::find()
            .filter(permission::Column::Name.eq(name))
            .one(db)
            .await?
        {
            perm.id
        } else {
            let perm_id = Uuid::new_v4();
            let new_perm = permission::ActiveModel {
                id: ActiveValue::Set(perm_id),
                name: ActiveValue::Set(name.to_string()),
                resource: ActiveValue::Set(resource.to_string()),
                action: ActiveValue::Set(action.to_string()),
                description: ActiveValue::Set(Some(description.to_string())),
            };
            Permission::insert(new_perm)
                .exec_without_returning(db)
                .await?;
            perm_id
        };
        permission_map.insert(name, perm_id);
    }

    // 4. Create Admin role
    let admin_role_id = if let Some(role) = Role::find()
        .filter(role::Column::Name.eq("Admin"))
        .filter(role::Column::OrganizationId.eq(default_org_id))
        .one(db)
        .await?
    {
        role.id
    } else {
        tracing::info!("Creating Admin role...");
        let role_id = Uuid::new_v4();
        let now = chrono::Utc::now().naive_utc();
        let new_role = role::ActiveModel {
            id: ActiveValue::Set(role_id),
            name: ActiveValue::Set("Admin".to_string()),
            description: ActiveValue::Set(Some("Full access".to_string())),
            organization_id: ActiveValue::Set(default_org_id),
            is_system_role: ActiveValue::Set(true),
            created_at: ActiveValue::Set(now),
        };
        Role::insert(new_role).exec_without_returning(db).await?;

        for perm_id in permission_map.values() {
            let role_perm = role_permission::ActiveModel {
                role_id: ActiveValue::Set(role_id),
                permission_id: ActiveValue::Set(*perm_id),
            };
            RolePermission::insert(role_perm)
                .exec_without_returning(db)
                .await?;
        }
        role_id
    };

    // 4b. Create Editor role
    if Role::find()
        .filter(role::Column::Name.eq("Editor"))
        .filter(role::Column::OrganizationId.eq(default_org_id))
        .one(db)
        .await?
        .is_none()
    {
        tracing::info!("Creating Editor role...");
        let role_id = Uuid::new_v4();
        let now = chrono::Utc::now().naive_utc();
        let new_role = role::ActiveModel {
            id: ActiveValue::Set(role_id),
            name: ActiveValue::Set("Editor".to_string()),
            description: ActiveValue::Set(Some("Can manage resources but not system settings".to_string())),
            organization_id: ActiveValue::Set(default_org_id),
            is_system_role: ActiveValue::Set(true),
            created_at: ActiveValue::Set(now),
        };
        Role::insert(new_role).exec_without_returning(db).await?;

        let editor_perms = [
            "organization.view",
            "members.view",
            "users.view",
            "users.create",
            "users.update",
        ];
        for perm_name in editor_perms {
            if let Some(perm_id) = permission_map.get(perm_name) {
                let role_perm = role_permission::ActiveModel {
                    role_id: ActiveValue::Set(role_id),
                    permission_id: ActiveValue::Set(*perm_id),
                };
                RolePermission::insert(role_perm)
                    .exec_without_returning(db)
                    .await?;
            }
        }
    }

    // 4c. Create Viewer role
    if Role::find()
        .filter(role::Column::Name.eq("Viewer"))
        .filter(role::Column::OrganizationId.eq(default_org_id))
        .one(db)
        .await?
        .is_none()
    {
        tracing::info!("Creating Viewer role...");
        let role_id = Uuid::new_v4();
        let now = chrono::Utc::now().naive_utc();
        let new_role = role::ActiveModel {
            id: ActiveValue::Set(role_id),
            name: ActiveValue::Set("Viewer".to_string()),
            description: ActiveValue::Set(Some("Read-only access".to_string())),
            organization_id: ActiveValue::Set(default_org_id),
            is_system_role: ActiveValue::Set(true),
            created_at: ActiveValue::Set(now),
        };
        Role::insert(new_role).exec_without_returning(db).await?;

        let viewer_perms = ["organization.view", "members.view", "users.view"];
        for perm_name in viewer_perms {
            if let Some(perm_id) = permission_map.get(perm_name) {
                let role_perm = role_permission::ActiveModel {
                    role_id: ActiveValue::Set(role_id),
                    permission_id: ActiveValue::Set(*perm_id),
                };
                RolePermission::insert(role_perm)
                    .exec_without_returning(db)
                    .await?;
            }
        }
    }

    // 5. Create admin user
    let admin_user_id = if let Some(existing_admin) = User::find()
        .filter(user::Column::Name.eq("admin@local.com"))
        .one(db)
        .await?
    {
        existing_admin.id
    } else {
        tracing::info!("Creating default admin user...");
        let salt = generate_salt();
        let hashed_password = hash_password("admin", &salt)?;
        let user_id = Uuid::new_v4();

        let admin_user = user::ActiveModel {
            id: ActiveValue::Set(user_id),
            name: ActiveValue::Set("admin@local.com".to_string()),
            password: ActiveValue::Set(hashed_password),
            salt: ActiveValue::Set(salt),
            email: ActiveValue::Set(Some("admin@local.com".to_string())),
            two_factor_secret: ActiveValue::NotSet,
            two_factor_enabled: ActiveValue::Set(false),
            force_password_change: ActiveValue::Set(true),
        };

        User::insert(admin_user).exec_without_returning(db).await?;
        tracing::info!("Admin user created: admin@local.com / admin (password change required)");
        user_id
    };

    // Ensure admin is in default org with Admin role
    if UserOrganization::find()
        .filter(user_organization::Column::UserId.eq(admin_user_id))
        .filter(user_organization::Column::OrganizationId.eq(default_org_id))
        .one(db)
        .await?
        .is_none()
    {
        let user_org = user_organization::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(admin_user_id),
            organization_id: ActiveValue::Set(default_org_id),
            role_id: ActiveValue::Set(admin_role_id),
            joined_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
        };
        UserOrganization::insert(user_org)
            .exec_without_returning(db)
            .await?;
    }

    tracing::info!("Database initialization completed");
    Ok(())
}
