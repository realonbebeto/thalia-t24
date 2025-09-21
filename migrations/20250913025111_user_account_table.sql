CREATE TYPE user_account_status AS ENUM ('active', 'closed', 'frozen', 'pending');
CREATE TABLE user_account (
    "id" UUID,
    "user_id" UUID NOT NULL,
    "account_number" VARCHAR(24) NOT NULL,
    "iban" VARCHAR(50) NOT NULL,
    "account_type" VARCHAR(64) NOT NULL,
    "coa_id" UUID NOT NULL,
    "branch_id" UUID NOT NULL,
    "currency" CHAR(3) NOT NULL,
    "status" user_account_status NOT NULL,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id),
    CONSTRAINT fk_user_account FOREIGN KEY(user_id) REFERENCES tuser(id) ON DELETE CASCADE
);