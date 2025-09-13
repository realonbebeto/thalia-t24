CREATE TYPE user_account_status AS ENUM ('active', 'closed', 'frozen');
CREATE TABLE user_account (
    "id" UUID,
    "user_id" UUID NOT NULL,
    "account_number" VARCHAR(24),
    "account_type" VARCHAR(64),
    "account_id" UUID,
    "branch_id" UUID,
    "currency" VARCHAR(5),
    "status" user_account_status NOT NULL,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id),
    CONSTRAINT fk_user_account FOREIGN KEY(user_id) REFERENCES user(id) ON DELETE CASCADE;
);