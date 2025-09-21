CREATE TABLE account_balance(
    "account_id" UUID,
    "amount_cents" BIGINT,
    PRIMARY KEY(account_id),
    CONSTRAINT fk_user_account FOREIGN KEY (account_id) REFERENCES user_account(id) ON DELETE CASCADE
)