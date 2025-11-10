use crate::staff::models::{ChartAccount, CoaType, CustomerAccountType};
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

pub struct StaffRepository<'a, 'b> {
    pool: &'a PgPool,
    tx: &'b mut Transaction<'a, Postgres>,
}

impl<'a, 'b> StaffRepository<'a, 'b> {
    pub fn from(pool: &'a PgPool, tx: &'b mut Transaction<'a, Postgres>) -> Self {
        Self { pool, tx }
    }

    #[tracing::instrument("Insert chart account to db", skip(self, coa))]
    pub async fn create_chart_account(&self, coa: &ChartAccount) -> Result<(), sqlx::Error> {
        sqlx::query(
        "INSERT INTO chart_of_account(id, code, name, coa_type, currency) VALUES($1, $2, $3, $4, $5)",
        )
        .bind(coa.get_id())
        .bind(coa.get_code())
        .bind(coa.get_name())
        .bind(coa.get_coa_type())
        .bind(coa.get_currency())
        .execute(self.pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument("Fetch chart account id by type from db", skip(self))]
    pub async fn fetch_coa_id_by_coa_type(
        &mut self,
        coa_type: CoaType,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let result: Option<Uuid> = sqlx::query("SELECT id FROM chart_of_account WHERE coa_type=$1")
            .bind(&coa_type)
            .fetch_optional(&mut **self.tx)
            .await?
            .map(|r| r.get("id"));

        Ok(result)
    }

    #[tracing::instrument("Insert account type in db", skip(self, acc_type))]
    pub async fn create_account_type(
        &self,
        acc_type: &CustomerAccountType,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO chart_of_account(id, name, description, coa_id) VALUES($1, $2, $3, $4)",
        )
        .bind(acc_type.id)
        .bind(&acc_type.name)
        .bind(&acc_type.description)
        .bind(acc_type.coa_id)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn fetch_coa_by_code(&self, code: &str) -> Result<Option<ChartAccount>, sqlx::Error> {
        let result = sqlx::query_as(
            "SELECT id, code, name, coa_type, currency FROM chart_of_account WHERE code=$1",
        )
        .bind(code)
        .fetch_optional(self.pool)
        .await?;

        Ok(result)
    }
}
