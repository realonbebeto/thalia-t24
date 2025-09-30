use chrono::NaiveDate;
use fake::{
    Fake,
    faker::internet::en,
    faker::name::en::{FirstName, LastName},
};
use getset::{Getters, Setters};
use sqlx::PgPool;
use thalia::base::{Email, Name, Password, Username};
use thalia::user::models::AccessRole;
use thalia::user::models::User;
use uuid::Uuid;

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
            date_of_birth: NaiveDate::from_ymd_opt(1994, 07, 11).unwrap(),
            email: Email::parse(en::SafeEmail().fake()).unwrap(),
            is_confirmed: false,
            is_active: false,
            is_verified: false,
            access_role: AccessRole::Superuser,
        };

        let customer = User {
            id: Uuid::now_v7(),
            first_name: Name::parse(FirstName().fake()).unwrap(),
            last_name: Name::parse(FirstName().fake()).unwrap(),
            username: Username::parse(en::Username().fake()).unwrap(),
            password: Password::parse(en::Password(std::ops::Range { start: 8, end: 16 }).fake())
                .unwrap(),
            date_of_birth: NaiveDate::from_ymd_opt(1994, 07, 11).unwrap(),
            email: Email::parse(en::SafeEmail().fake()).unwrap(),
            is_confirmed: false,
            is_active: false,
            is_verified: false,
            access_role: AccessRole::Customer,
        };

        Self { staff, customer }
    }

    pub async fn store_test_users(&self, pool: &PgPool) {
        sqlx::query(
            "INSERT INTO tuser(id, first_name, last_name, username, password, date_of_birth, email, is_confirmed, is_active, is_verified, access_role)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11), ($12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)",
        )
        .bind(self.get_staff().get_id())
        .bind(self.get_staff().get_first_name().as_ref())
        .bind(self.get_staff().get_last_name().as_ref())
        .bind(self.get_staff().get_username().as_ref())
        .bind(self.get_staff().get_password().phash_as_ref())
        .bind(self.get_staff().get_date_of_birth())
        .bind(self.get_staff().get_email().as_ref())
        .bind(self.get_staff().get_is_confirmed())
        .bind(self.get_staff().get_is_active())
        .bind(self.get_staff().get_is_verified())
        .bind(AccessRole::Superuser)
        // Customer
        .bind(self.get_customer().get_id())
        .bind(self.get_customer().get_first_name().as_ref())
        .bind(self.get_customer().get_last_name().as_ref())
        .bind(self.get_customer().get_username().as_ref())
        .bind(self.get_customer().get_password().phash_as_ref())
        .bind(self.get_customer().get_date_of_birth())
        .bind(self.get_customer().get_email().as_ref())
        .bind(self.get_customer().get_is_confirmed())
        .bind(self.get_customer().get_is_active())
        .bind(self.get_customer().get_is_verified())
        .bind(AccessRole::Customer)
        .execute(pool).await.expect("Failed to store test users");
    }
}
