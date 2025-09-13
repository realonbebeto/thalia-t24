CREATE TYPE user_role AS ENUM ('superuser', 'manager', 'customer');
CREATE TABLE user (
    "id" UUID,
    "first_name" VARCHAR(64),
    "last_name" VARCHAR(64),
    "email" TEXT NOT NULL UNIQUE,
    "is_active" BOOLEAN NOT NULL,
    "access_role" user_role NOT NULL,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id)
);