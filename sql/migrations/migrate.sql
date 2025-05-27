-- Migration script to create the initial database schema for the application.
\set ON_ERROR_STOP on
\! echo '\033[0;33mMigrating Database\033[0m';

-- Please insert your SQL file below to be executed in migration process.
\i sql/migrations/001_create_donations.sql

\i sql/migrations/enable_dblink.sql
\i sql/migrations/notification.sql
\i sql/migrations/campaigns.sql
\i sql/migrations/wallet.sql
\i sql/migrations/transaction.sql
-- \i sql/migrations/other_table.sql