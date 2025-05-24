-- Notification Migration Script
\! echo '\033[33mMigrating Notification Database\033[0m';

-- Drop existing tables if they exist
DROP TABLE IF EXISTS notification_user;
DROP TABLE IF EXISTS announcement_subscription;
DROP TABLE IF EXISTS notification;

-- Notification table
CREATE TABLE IF NOT EXISTS notification (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content VARCHAR(255) NOT NULL,
    created_at timestamp NOT NULL DEFAULT NOW(),
    target_type VARCHAR(255) NOT NULL
        DEFAULT 'AllUsers'
        CHECK (target_type IN (
            'AllUsers',
            'SpecificUser',
            'Fundraisers',
            'Donors',
            'NewCampaign'
        )
    )
);

-- New Campaign Announcement Subscription table
CREATE TABLE IF NOT EXISTS announcement_subscription (
    user_email VARCHAR(255) NOT NULL PRIMARY KEY,
    start_at timestamp NOT NULL DEFAULT NOW()
);

-- Announcement for User table
CREATE TABLE IF NOT EXISTS notification_user (
    user_email VARCHAR(255) NOT NULL,
    announcement_id INT NOT NULL,
    created_at timestamp NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_email, announcement_id),
    FOREIGN KEY (announcement_id) REFERENCES notification(id)
);