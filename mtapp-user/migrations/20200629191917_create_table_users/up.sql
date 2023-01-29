CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  username VARCHAR NOT NULL,
  email VARCHAR,
  password VARCHAR NOT NULL,
  last_logged_in_at Timestamp WITH TIME ZONE,
  created_at Timestamp WITH TIME ZONE NOT NULL DEFAULT now(),
  updated_at Timestamp WITH TIME ZONE NOT NULL DEFAULT now(),
  CONSTRAINT username_uniq UNIQUE (username),
  CONSTRAINT email_uniq UNIQUE (email)
);

CREATE TRIGGER user_updated BEFORE INSERT OR UPDATE ON users
FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
