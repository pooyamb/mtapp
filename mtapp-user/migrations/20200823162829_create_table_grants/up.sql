CREATE TABLE IF NOT EXISTS grants (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    scope_id UUID NOT NULL,
    created_at Timestamp WITH TIME ZONE NOT NULL DEFAULT now(),
    CONSTRAINT grants_user_id FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,
    CONSTRAINT grants_scope_id FOREIGN KEY (scope_id) REFERENCES scopes (id) ON DELETE CASCADE,
    CONSTRAINT grants_uniq UNIQUE (user_id, scope_id)
);