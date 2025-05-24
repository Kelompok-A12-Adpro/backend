-- Migration script to create the test schema database schema for the application.
\! echo '\033[0;33mMigrating Test Database\033[0m';

-- Please insert your SQL file below to be executed in migration process.
\i sql/test_migrations/notification.sql;
-- \i sql/test_migrations/name.sql;