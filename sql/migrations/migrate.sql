-- Migration script to create the initial database schema for the application.
\! echo '\033[0;33mMigrating Database\033[0m';

-- Please insert your SQL file below to be executed in migration process.
\i sql/migrations/notification.sql
-- \i sql/migrations/other_table.sql