CREATE TABLE journal_entry(
    "id" UUID,
    "reference_id" VARCHAR(50) UNIQUE,
    "description" TEXT,
    "created_date" TIMESTAMP NOT NULL DEFAULT NOW(),
    PRIMARY KEY(id)
);