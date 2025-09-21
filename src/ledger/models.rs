use getset::Getters;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, serde::Deserialize, Getters)]
#[get = "pub with_prefix"]
pub struct JournalEntry {
    id: Uuid,
    user_account_id: Uuid,
    transaction_id: String,
    transaction_ref: String,
    description: String,
}

impl JournalEntry {
    pub fn new(
        user_account_id: Uuid,
        transaction_id: String,
        transaction_ref: String,
        description: String,
    ) -> Self {
        JournalEntry {
            id: Uuid::now_v7(),
            user_account_id,
            transaction_id,
            transaction_ref,
            description,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::Type)]
#[sqlx(type_name = "ledger_line_type", rename_all = "lowercase")]
pub enum LineType {
    Credit,
    Debit,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, Getters)]
#[get = "pub with_prefix"]
pub struct JournalLine {
    id: Uuid,
    journal_entry_id: Uuid,
    coa_id: Uuid,
    amount_cents: i32,
    line_type: LineType,
}

#[derive(Getters)]
#[get = "pub with_prefix"]
pub struct DebitLine {
    id: Uuid,
    coa_id: Uuid,
    line_type: LineType,
}

impl DebitLine {
    pub fn new(coa_id: Uuid, line_type: LineType) -> Self {
        DebitLine {
            id: Uuid::now_v7(),
            coa_id,
            line_type,
        }
    }
}

#[derive(Getters)]
#[get = "pub with_prefix"]
pub struct CreditLine {
    id: Uuid,
    coa_id: Uuid,
    line_type: LineType,
}

impl CreditLine {
    pub fn new(coa_id: Uuid, line_type: LineType) -> Self {
        CreditLine {
            id: Uuid::now_v7(),
            coa_id,
            line_type,
        }
    }
}

#[derive(Getters)]
#[get = "pub with_prefix"]
pub struct IntoJournalLine {
    journal_entry_id: Uuid,
    amount_cents: i64,
    debit_line: DebitLine,
    credit_line: CreditLine,
}

impl IntoJournalLine {
    pub fn new(
        journal_entry_id: Uuid,
        amount: f64,
        debit_line: DebitLine,
        credit_line: CreditLine,
    ) -> Self {
        IntoJournalLine {
            amount_cents: (amount * 100.0) as i64,
            journal_entry_id,
            debit_line,
            credit_line,
        }
    }
}
