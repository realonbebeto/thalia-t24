BEGIN;
CREATE TYPE user_role AS ENUM ('superuser', 'manager', 'customer');
CREATE TABLE tuser (
    "id" UUID,
    "first_name" VARCHAR(64),
    "last_name" VARCHAR(64),
    "username" TEXT NOT NULL UNIQUE,
    "date_of_birth" DATE NOT NULL,
    "email" TEXT NOT NULL UNIQUE,
    "is_active" BOOLEAN NOT NULL,
    "is_verified" BOOLEAN NOT NULL,
    "access_role" user_role NOT NULL,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id)
);
COMMIT;