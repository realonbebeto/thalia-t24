use sqlx::PgPool;
use std::{collections::HashMap, fs::File};

use thalia::account::models::{AccountClass, AccountKind, BehaviorPolicy};
use thalia::staff::models::ChartAccount;
use uuid::Uuid;

#[derive(Debug, getset::Setters, getset::Getters)]
#[get = "pub with_prefix"]
pub struct AccountClasses {
    checking: AccountClass,
    saving: AccountClass,
    loan: AccountClass,
}

#[derive(Debug, getset::Setters, getset::Getters)]
#[get = "pub with_prefix"]
pub struct Coas {
    store: HashMap<String, ChartAccount>,
}

impl Coas {
    pub fn default() -> Self {
        let file = File::open("assets/coa.csv").unwrap();
        let mut rdr = csv::Reader::from_reader(file);

        let mut store: HashMap<String, ChartAccount> = HashMap::new();

        for result in rdr.records() {
            let record = result.unwrap();
            let coa = ChartAccount::new(
                Uuid::now_v7(),
                record.get(0).unwrap(),
                record.get(1).unwrap(),
                record.get(2).unwrap().try_into().unwrap(),
                record.get(3).unwrap(),
            );

            store.insert(record.get(0).unwrap().to_string(), coa);
        }

        Self { store }
    }

    async fn store_coa(&self, pool: &PgPool, coa: &ChartAccount) {
        sqlx::query("INSERT INTO chart_of_account(id, code, name, coa_type, currency) VALUES($1, $2, $3, $4, $5)")
            .bind(coa.get_id())
            .bind(coa.get_code())
            .bind(coa.get_name())
            .bind(coa.get_coa_type())
            .bind(coa.get_currency())
            .execute(pool)
            .await
            .expect("Failed to insert coa");
    }

    pub async fn store_coas(&self, pool: &PgPool) {
        for v in self.store.values() {
            self.store_coa(pool, v).await;
        }
    }
}

impl AccountClasses {
    pub fn default(coas: &Coas) -> Self {
        let behave_policy = BehaviorPolicy::new(575, 100);
        let checking = AccountClass::new(
            Uuid::now_v7(),
            AccountKind::Deposit,
            "1000",
            "Checking",
            "Transaction Account",
            *coas.get_store().get("2010").unwrap().get_id(),
            &behave_policy,
        );

        let saving = AccountClass::new(
            Uuid::now_v7(),
            AccountKind::Deposit,
            "1001",
            "Saving",
            "Saving Account",
            *coas.get_store().get("2020").unwrap().get_id(),
            &behave_policy,
        );

        let loan = AccountClass::new(
            Uuid::now_v7(),
            AccountKind::Loan,
            "1002",
            "Loan",
            "Loan Account",
            *coas.get_store().get("1400").unwrap().get_id(),
            &behave_policy,
        );

        Self {
            checking,
            saving,
            loan,
        }
    }

    async fn store_account_class(&self, pool: &PgPool, class: &AccountClass) {
        sqlx::query("INSERT INTO account_class(id, kind, code, name, description, coa_id, default_interest_rate, default_min_balance) VALUES($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind(class.get_id())
            .bind(class.get_kind())
            .bind(class.get_code())
            .bind(class.get_name())
            .bind(class.get_description())
            .bind(class.get_coa_id())
            .bind(*class.get_default_interest_rate() as i32)
            .bind(*class.get_default_min_balance() as i32 )
            .execute(pool).await.expect("Failed to insert account class");
    }

    pub async fn store_account_classes(&self, pool: &PgPool) {
        self.store_account_class(pool, &self.checking).await;
        self.store_account_class(pool, &self.saving).await;
        self.store_account_class(pool, &self.loan).await;
    }
}
