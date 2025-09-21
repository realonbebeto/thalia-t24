CREATE TABLE journal_entry(
    "id" UUID,
    "user_account_id" UUID NOT NULL,
    "transaction_id" VARCHAR(50) UNIQUE,
    "transaction_ref" VARCHAR(50) UNIQUE,
    "description" TEXT,
    "created_date" TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY(id),
    CONSTRAINT fk_user_account FOREIGN KEY(user_account_id) REFERENCES user_account(id)
);