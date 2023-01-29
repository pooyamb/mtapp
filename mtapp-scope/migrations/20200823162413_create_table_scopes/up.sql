CREATE EXTENSION IF NOT EXISTS "uuid-ossp" SCHEMA public;

CREATE TABLE IF NOT EXISTS scopes (
  id UUID PRIMARY KEY,
  name VARCHAR NOT NULL,
  created_at Timestamp WITH TIME ZONE NOT NULL DEFAULT now(),
  CONSTRAINT name_uniq UNIQUE (name)
);

INSERT INTO
  scopes (id, name)
VALUES
  (uuid_generate_v4(), 'superuser'),
  (uuid_generate_v4(), 'admin'),
  (uuid_generate_v4(), 'confirmed'),
  (uuid_generate_v4(), 'active');