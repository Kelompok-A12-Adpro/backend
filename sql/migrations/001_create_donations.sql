-- Drop the table if it exists to ensure a clean slate (as per instructions)
\! echo '\033[33mMigrating Donations Database\033[0m';
DROP TABLE IF EXISTS donations CASCADE; -- Use CASCADE if other tables reference it

-- Create the donations table
CREATE TABLE donations (
    id SERIAL PRIMARY KEY,                            -- Corresponds to pub id: i32
    user_id INT NOT NULL,                           -- Corresponds to pub user_id: i32
    campaign_id INT NOT NULL,                       -- Corresponds to pub campaign_id: i32
    amount BIGINT NOT NULL,               -- Corresponds to pub amount: f64
    message TEXT,                                   -- Corresponds to pub message: Option<String>
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP -- Corresponds to pub created_at: DateTime<Utc>

    -- Optional: Add foreign key constraints if users and campaigns tables exist
    -- CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    -- CONSTRAINT fk_campaign FOREIGN KEY(campaign_id) REFERENCES campaigns(id) ON DELETE CASCADE
);

-- Optional: Add indexes for frequently queried columns
CREATE INDEX IF NOT EXISTS idx_donations_user_id ON donations(user_id);
CREATE INDEX IF NOT EXISTS idx_donations_campaign_id ON donations(campaign_id);

\echo 'Created donations table'