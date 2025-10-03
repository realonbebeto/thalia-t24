BEGIN;
CREATE TYPE chart_account_type AS ENUM (
    'asset',
    'equity',
    'expense',
    'income',
    'liability',
    'memoranda'
);
CREATE TABLE chart_of_account(
    "id" UUID,
    "code" VARCHAR(20) UNIQUE NOT NULL,
    "name" VARCHAR(64) NOT NULL,
    "coa_type" chart_account_type NOT NULL,
    "currency" VARCHAR(5) NOT NULL,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id)
);
COMMIT;