BEGIN;
CREATE TYPE account_kind AS ENUM ('deposit', 'investment', 'loan', 'specialty');
CREATE TABLE account_class (
    "id" UUID NOT NULL,
    "code" VARCHAR(20) NOT NULL,
    "kind" VARCHAR(20) NOT NULL,
    "name" VARCHAR(100) NOT NULL,
    "description" TEXT,
    "coa_id" UUID NOT NULL,
    "default_interest_rate" INT,
    "default_min_balance" INT,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_chart_account FOREIGN KEY(coa_id) REFERENCES chart_of_account(id)
);
COMMIT;