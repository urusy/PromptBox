-- Smart folders table
CREATE TABLE IF NOT EXISTS smart_folders (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    icon VARCHAR(50),
    filters JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Index for listing (order by created_at)
CREATE INDEX IF NOT EXISTS idx_smart_folders_created_at ON smart_folders(created_at DESC);
