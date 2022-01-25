CREATE TABLE IF NOT EXISTS user_vc_tplt_selections (
    user_id UUID PRIMARY KEY NOT NULL,
    tplt_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);