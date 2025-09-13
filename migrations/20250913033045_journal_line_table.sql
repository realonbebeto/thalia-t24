CREATE TABLE journal_line (
    "id" UUID,
    "journal_entry_id" UUID,
    "coa_id" UUID,
    "amount_cents" BIGINT NOT NULL CHECK (amount_cents > 0),
    "line_type" VARCHAR(6) NOT NULL CHECK (line_type IN ('DEBIT', 'CREDIT')),
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id),
    CONSTRAINT fk_journal_entry FOREIGN KEY(journal_entry_id) REFERENCES journal_entry(id) ON DELETE CASCADE,
    CONSTRAINT fk_chart_of_account FOREIGN KEY(coa_id) REFERENCES chart_of_account(id) ON DELETE CASCADE
);