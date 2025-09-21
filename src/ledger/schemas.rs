use itertools::Itertools;
use sqlx::types::chrono;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JournalRequest {
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub reference_id: Option<String>,
    pub journal_id: Option<Vec<Uuid>>,
    pub line_type: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct JournalEntryLine {
    id: Uuid,
    reference_id: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    coa_id: Uuid,
    line_type: String,
    amount_cents: i64,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct JournalLine {
    coa_id: Uuid,
    line_type: String,
    amount_cents: u64,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct JEntryLineResponse {
    id: Uuid,
    reference_id: String,
    description: String,
    created_at: chrono::DateTime<chrono::Utc>,
    lines: Vec<JournalLine>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct JournalResponse {
    entries: Vec<JEntryLineResponse>,
}

impl From<Vec<JournalEntryLine>> for JournalResponse {
    fn from(value: Vec<JournalEntryLine>) -> Self {
        value
            .into_iter()
            .chunk_by(|row| row.id)
            .into_iter()
            .map(|(id, group)| {
                let group: Vec<JournalEntryLine> = group.collect();
                let first = &group[0];

                JEntryLineResponse {
                    id,
                    reference_id: first.reference_id.clone(),
                    description: first.description.clone(),
                    created_at: first.created_at,
                    lines: group
                        .into_iter()
                        .map(|row| JournalLine {
                            coa_id: row.coa_id,
                            line_type: row.line_type,
                            amount_cents: row.amount_cents as u64,
                        })
                        .collect(),
                }
            })
            .collect::<JournalResponse>()
    }
}

impl FromIterator<JEntryLineResponse> for JournalResponse {
    fn from_iter<T: IntoIterator<Item = JEntryLineResponse>>(iter: T) -> Self {
        let mut jr = JournalResponse {
            entries: Vec::new(),
        };

        for i in iter {
            jr.entries.push(i)
        }

        jr
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JournalIdRequest {
    pub journal_id: Uuid,
}
