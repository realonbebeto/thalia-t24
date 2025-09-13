CREATE TABLE user_account_balances (
    user_account_id UUID,
    balance_cents BIGINT NOT NULL DEFAULT 0,
    updated_at timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(user_account_id),
    CONSTRAINT fk_user_account FOREIGN KEY(user_account_id) REFERENCES user_account(id) On DELETE CASCADE
);