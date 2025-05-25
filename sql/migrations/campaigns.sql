\! echo '\033[33mMigrating Campaigns Table + CRUD Templates\033[0m';

-- DROP & CREATE table
DROP TABLE IF EXISTS campaigns CASCADE;

CREATE TABLE IF NOT EXISTS campaigns (
  id                    SERIAL PRIMARY KEY,
  user_id               INT            NOT NULL,
  name                  VARCHAR(255)   NOT NULL,
  description           TEXT           NOT NULL,
  target_amount         BIGINT        NOT NULL,
  collected_amount      BIGINT        NOT NULL DEFAULT 0,
  start_date            TIMESTAMP      NOT NULL,
  end_date              TIMESTAMP      NOT NULL,
  image_url             VARCHAR(255),
  status                VARCHAR(32)    NOT NULL
                           DEFAULT 'PendingVerification'
                           CHECK (status IN (
                             'PendingVerification','Active','Completed','Rejected'
                           )),
  evidence_url          VARCHAR(255),
  evidence_uploaded_at  TIMESTAMP,
  created_at            TIMESTAMP      NOT NULL DEFAULT NOW(),
  updated_at            TIMESTAMP      NOT NULL DEFAULT NOW()
);