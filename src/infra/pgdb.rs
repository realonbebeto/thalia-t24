use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    account::repo::AccountRepository, authentication::repo::AuthRepository,
    ledger::repo::LedgerRepository, staff::repo::StaffRepository,
    transaction::repo::TransactionRepository, user::repo::UserRepository,
};

pub struct UnitofWork<'a> {
    pool: &'a PgPool,
    tx: Transaction<'a, Postgres>,
}

impl<'a> UnitofWork<'a> {
    pub async fn from(pool: &'a PgPool) -> Result<Self, sqlx::Error> {
        let tx = pool.begin().await?;

        Ok(Self { pool, tx })
    }

    pub async fn commit(self) -> Result<(), sqlx::Error> {
        self.tx.commit().await
    }

    pub fn authentication(&mut self) -> AuthRepository<'a, '_> {
        AuthRepository::from(self.pool, &mut self.tx)
    }

    pub fn users(&mut self) -> UserRepository<'a, '_> {
        UserRepository::from(self.pool, &mut self.tx)
    }

    pub fn accounts(&mut self) -> AccountRepository<'a, '_> {
        AccountRepository::from(self.pool, &mut self.tx)
    }

    pub fn ledgers(&mut self) -> LedgerRepository<'a, '_> {
        LedgerRepository::from(self.pool, &mut self.tx)
    }

    pub fn staffs(&mut self) -> StaffRepository<'a, '_> {
        StaffRepository::from(self.pool, &mut self.tx)
    }
    pub fn transactions(&mut self) -> TransactionRepository<'a, '_> {
        TransactionRepository::from(self.pool, &mut self.tx)
    }
}
