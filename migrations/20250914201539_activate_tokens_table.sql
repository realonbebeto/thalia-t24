CREATE TABLE activate_token(
    "token" VARCHAR(256) NOT NULL,
    "user_id" UUID NOT NULL,
    "user_email" TEXT NOT NULL,
    PRIMARY KEY(token),
    CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES tuser(id) ON DELETE CASCADE
);