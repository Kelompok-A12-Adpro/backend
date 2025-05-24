-- Run this one if you want to fully re-initialize the database
-- Please add your DDL query file on both migration folder (if you use database for testing),
-- and include it on the correlated migration.sql

-- Database initialization script
\set ON_ERROR_STOP on -- Add this line
\! echo '\033[1;32mStarting database initialization\033[0m';

-- Include migration
\i sql/migrations/migrate.sql


-- Create test database if not exist
\! echo '\033[1;32mTrying to create test database\033[0m';
DO
$$
BEGIN
   IF EXISTS (SELECT FROM pg_database WHERE datname = 'gatherlove_test') THEN
      RAISE NOTICE 'Test Database already exists';
   ELSE
        PERFORM dblink_exec(
            'dbname=' || current_database(),
            'CREATE DATABASE gatherlove_test'
        );
        RAISE NOTICE 'Test Database created';
   END IF;
END
$$;

-- Connect to the test database
\c gatherlove_test;

-- Include test migrations
\i sql/migrations/migrate.sql

-- Finish
\! echo '\033[1;32mDatabase successfully migrated\033[0m';