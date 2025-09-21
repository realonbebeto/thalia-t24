BEGIN;
CREATE TYPE header_pair AS (name TEXT, value BYTEA);
CREATE TABLE transaction_idempotent (
    "account_id" UUID,
    "transaction_ref" VARCHAR(50) NOT NULL,
    "amount_cents" BIGINT NOT NULL CHECK (amount_cents > 0),
    "response_status_code" SMALLINT,
    "response_headers" header_pair [],
    "response_body" BYTEA,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(account_id, transaction_ref),
    CONSTRAINT fk_tx_idempotency FOREIGN KEY(account_id) REFERENCES user_account(id) ON DELETE CASCADE
);
COMMIT;