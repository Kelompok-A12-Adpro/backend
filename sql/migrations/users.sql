\! echo '\033[33mMigrating Users Table\033[0m';

-- drop dulu kalau sudah ada
DROP TABLE IF EXISTS users CASCADE;

-- buat table users
CREATE TABLE IF NOT EXISTS users (
  id          SERIAL PRIMARY KEY,
  email       VARCHAR(255) NOT NULL UNIQUE,
  password    VARCHAR(255) NOT NULL,
  name        VARCHAR(100) NOT NULL,
  phone       VARCHAR(20)  NOT NULL,
  is_admin    BOOLEAN      NOT NULL DEFAULT FALSE
);  