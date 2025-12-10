-- search_presetsテーブル
CREATE TABLE search_presets (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    filters JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- テーブルコメント
COMMENT ON TABLE search_presets IS '検索条件プリセット';
COMMENT ON COLUMN search_presets.id IS 'UUID v7（時系列ソート可能）';
COMMENT ON COLUMN search_presets.name IS 'プリセット名';
COMMENT ON COLUMN search_presets.filters IS '検索条件（JSONB）';

-- インデックス
CREATE INDEX idx_search_presets_created_at ON search_presets(created_at DESC);

-- updated_at自動更新トリガー
CREATE TRIGGER trigger_search_presets_updated_at
    BEFORE UPDATE ON search_presets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
