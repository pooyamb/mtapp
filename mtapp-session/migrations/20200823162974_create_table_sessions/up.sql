CREATE TABLE IF NOT EXISTS sessions (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL,
  ip VARCHAR NOT NULL,
  user_agent VARCHAR NOT NULL,
  jti UUID NOT NULL,
  refresh_token UUID NOT NULL,
  last_access_at Timestamp WITH TIME ZONE NOT NULL DEFAULT now(),
  created_at Timestamp WITH TIME ZONE NOT NULL DEFAULT now(),
  CONSTRAINT refresh_token UNIQUE (refresh_token),
  CONSTRAINT jti_uniq UNIQUE (jti),
  CONSTRAINT sessions_user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);