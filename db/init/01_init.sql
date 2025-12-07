-- 拡張機能
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- imagesテーブル
CREATE TABLE images (
    id UUID PRIMARY KEY,
    source_tool VARCHAR(20) NOT NULL,
    model_type VARCHAR(20),
    has_metadata BOOLEAN NOT NULL DEFAULT TRUE,
    original_filename VARCHAR(255) NOT NULL,
    storage_path VARCHAR(512) NOT NULL,
    thumbnail_path VARCHAR(512) NOT NULL,
    file_hash VARCHAR(64) NOT NULL UNIQUE,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    positive_prompt TEXT,
    negative_prompt TEXT,
    model_name VARCHAR(255),
    sampler_name VARCHAR(50),
    scheduler VARCHAR(50),
    steps INTEGER,
    cfg_scale NUMERIC(5,2),
    seed BIGINT,
    loras JSONB NOT NULL DEFAULT '[]'::jsonb,
    controlnets JSONB NOT NULL DEFAULT '[]'::jsonb,
    embeddings JSONB NOT NULL DEFAULT '[]'::jsonb,
    model_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    workflow_extras JSONB NOT NULL DEFAULT '{}'::jsonb,
    raw_metadata JSONB,
    rating SMALLINT NOT NULL DEFAULT 0,
    is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
    needs_improvement BOOLEAN NOT NULL DEFAULT FALSE,
    user_tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    user_memo TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT rating_range CHECK (rating >= 0 AND rating <= 5),
    CONSTRAINT valid_dimensions CHECK (width > 0 AND height > 0),
    CONSTRAINT valid_file_size CHECK (file_size_bytes > 0)
);

-- テーブルコメント
COMMENT ON TABLE images IS 'AI生成画像のメタデータと管理情報';
COMMENT ON COLUMN images.id IS 'UUID v7（時系列ソート可能）';
COMMENT ON COLUMN images.source_tool IS '生成ツール（comfyui/a1111/forge/novelai/unknown）';
COMMENT ON COLUMN images.model_type IS 'モデルタイプ（sd15/sdxl/pony/illustrious/flux/qwen/other）';
COMMENT ON COLUMN images.file_hash IS 'SHA-256ハッシュ（重複チェック用）';
COMMENT ON COLUMN images.model_params IS 'モデル固有パラメータ（JSONB）';
COMMENT ON COLUMN images.deleted_at IS '論理削除日時（NULLは未削除）';

-- インデックス
CREATE INDEX idx_images_source_tool ON images(source_tool);
CREATE INDEX idx_images_model_type ON images(model_type);
CREATE INDEX idx_images_model_name ON images(model_name);
CREATE INDEX idx_images_sampler_name ON images(sampler_name);
CREATE INDEX idx_images_steps ON images(steps);
CREATE INDEX idx_images_cfg_scale ON images(cfg_scale);
CREATE INDEX idx_images_rating ON images(rating);
CREATE INDEX idx_images_created_at ON images(created_at DESC);
CREATE INDEX idx_images_is_favorite ON images(is_favorite) WHERE is_favorite = TRUE;
CREATE INDEX idx_images_needs_improvement ON images(needs_improvement) WHERE needs_improvement = TRUE;
CREATE INDEX idx_images_deleted ON images(deleted_at) WHERE deleted_at IS NOT NULL;
CREATE INDEX idx_images_model_name_trgm ON images USING gin(model_name gin_trgm_ops);
CREATE INDEX idx_images_original_filename_trgm ON images USING gin(original_filename gin_trgm_ops);
CREATE INDEX idx_images_positive_prompt_fts ON images USING gin(to_tsvector('english', COALESCE(positive_prompt, '')));
CREATE INDEX idx_images_negative_prompt_fts ON images USING gin(to_tsvector('english', COALESCE(negative_prompt, '')));
CREATE INDEX idx_images_loras ON images USING gin(loras jsonb_path_ops);
CREATE INDEX idx_images_controlnets ON images USING gin(controlnets jsonb_path_ops);
CREATE INDEX idx_images_user_tags ON images USING gin(user_tags jsonb_path_ops);
CREATE INDEX idx_images_model_params ON images USING gin(model_params jsonb_path_ops);
CREATE INDEX idx_images_list_default ON images(created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_images_model_rating ON images(model_type, rating DESC) WHERE deleted_at IS NULL;

-- updated_at自動更新トリガー
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_images_updated_at
    BEFORE UPDATE ON images
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();