use fake::{
    Fake,
    faker::chrono::en::Date,
    faker::internet::en,
    faker::name::en::{FirstName, LastName},
};
use getset::{Getters, Setters};
use sqlx::PgPool;
use thalia::base::{Email, Name, Password, Username};
use thalia::user::models::AccessRole;
use uuid::Uuid;

#[derive(Debug, Setters, Getters)]
#[get = "pub with_prefix"]
pub struct User {
    id: Uuid,
    first_name: Name,
    last_name: Name,
    username: Username,
    password: Password,
    date_of_birth: chrono::NaiveDate,
    email: Email,
    is_active: bool,
    is_verified: bool,
    access_role: AccessRole,
}

#[derive(Debug, Setters, Getters)]
#[get = "pub with_prefix"]
pub struct TestUsers {
    staff: User,
    customer: User,
}

impl TestUsers {
    pub fn generate_users() -> Self {
        let staff = User {
            id: Uuid::now_v7(),
            first_name: Name::parse(FirstName().fake()).unwrap(),
            last_name: Name::parse(LastName().fake()).unwrap(),
            username: Username::parse(en::Username().fake()).unwrap(),
            password: Password::parse(en::Password(std::ops::Range { start: 8, end: 16 }).fake())
                .unwrap(),
            date_of_birth: Date().fake(),
            email: Email::parse(en::SafeEmail().fake()).unwrap(),
            is_active: true,
            is_verified: true,
            access_role: AccessRole::Superuser,
        };

        let customer = User {
            id: Uuid::now_v7(),
            first_name: Name::parse(FirstName().fake()).unwrap(),
            last_name: Name::parse(FirstName().fake()).unwrap(),
            username: Username::parse(en::Username().fake()).unwrap(),
            password: Password::parse(en::Password(std::ops::Range { start: 8, end: 16 }).fake())
                .unwrap(),
            date_of_birth: Date().fake(),
            email: Email::parse(en::SafeEmail().fake()).unwrap(),
            is_active: true,
            is_verified: true,
            access_role: AccessRole::Customer,
        };

        Self { staff, customer }
    }

    pub async fn store_test_users(&self, pool: &PgPool) {
        sqlx::query(
            "INSERT INTO tuser(id, first_name, last_name, username, password, date_of_birth, email, is_active, is_verified, access_role)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10), ($11, $12, $13, $14, $15, $16, $17, $18, $19, $20)",
        )
        .bind(self.get_staff().get_id())
        .bind(self.get_staff().get_first_name().as_ref())
        .bind(self.get_staff().get_last_name().as_ref())
        .bind(self.get_staff().get_username().as_ref())
        .bind(self.get_staff().get_password().phash_as_ref())
        .bind(self.get_staff().get_date_of_birth())
        .bind(self.get_staff().get_email().as_ref())
        .bind(true)
        .bind(true)
        .bind(AccessRole::Superuser)
        // Customer
        .bind(self.get_customer().get_id())
        .bind(self.get_customer().get_first_name().as_ref())
        .bind(self.get_customer().get_last_name().as_ref())
        .bind(self.get_customer().get_username().as_ref())
        .bind(self.get_customer().get_password().phash_as_ref())
        .bind(self.get_customer().get_date_of_birth())
        .bind(self.get_customer().get_email().as_ref())
        .bind(true)
        .bind(true)
        .bind(AccessRole::Customer)
        .execute(pool).await.expect("Failed to store test users");
    }
}
