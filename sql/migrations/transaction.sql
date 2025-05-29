DROP TABLE IF EXISTS transactions CASCADE;
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    wallet_id INTEGER NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    transaction_type VARCHAR NOT NULL, -- e.g. "topup", "donation", "withdrawal"
    amount DOUBLE PRECISION NOT NULL,
    method VARCHAR,                   -- nullable, e.g. "GOPAY", "DANA"
    phone_number VARCHAR,            -- nullable phone number
    campaign_id INTEGER,             -- nullable, for donations
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE  -- soft delete flag
);
