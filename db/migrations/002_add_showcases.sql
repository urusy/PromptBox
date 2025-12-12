-- Migration: 002_add_showcases
-- Description: Add showcases and showcase_images tables
-- Date: 2025-01-XX (adjust as needed)

-- Check if migration is needed (idempotent)
DO $$
BEGIN
    -- Create showcases table if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'showcases') THEN
        CREATE TABLE showcases (
            id UUID PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            description TEXT,
            icon VARCHAR(50),
            cover_image_id UUID REFERENCES images(id) ON DELETE SET NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );

        COMMENT ON TABLE showcases IS '画像のカスタムコレクション';
        COMMENT ON COLUMN showcases.cover_image_id IS 'カバー画像（カード表示用）';

        CREATE INDEX idx_showcases_created_at ON showcases(created_at DESC);

        CREATE TRIGGER trigger_showcases_updated_at
            BEFORE UPDATE ON showcases
            FOR EACH ROW
            EXECUTE FUNCTION update_updated_at_column();

        RAISE NOTICE 'Created table: showcases';
    ELSE
        RAISE NOTICE 'Table showcases already exists, skipping';
    END IF;

    -- Create showcase_images table if not exists
    IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'showcase_images') THEN
        CREATE TABLE showcase_images (
            showcase_id UUID NOT NULL REFERENCES showcases(id) ON DELETE CASCADE,
            image_id UUID NOT NULL REFERENCES images(id) ON DELETE CASCADE,
            sort_order INTEGER NOT NULL DEFAULT 0,
            added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (showcase_id, image_id)
        );

        COMMENT ON TABLE showcase_images IS 'Showcaseと画像の関連（順序付き）';
        COMMENT ON COLUMN showcase_images.sort_order IS '表示順序';

        CREATE INDEX idx_showcase_images_showcase ON showcase_images(showcase_id, sort_order);
        CREATE INDEX idx_showcase_images_image ON showcase_images(image_id);

        RAISE NOTICE 'Created table: showcase_images';
    ELSE
        RAISE NOTICE 'Table showcase_images already exists, skipping';
    END IF;
END
$$;
