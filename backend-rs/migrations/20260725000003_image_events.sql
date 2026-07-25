-- A1: change feed with tombstones (docs/13 §3 観点①).
--
-- Falcon imports images once and skips anything it has already seen
-- (`files.source_url`), so today an image is transferred exactly once and never
-- updated again: a rating changed here never reaches Falcon, and a deletion
-- never propagates at all. Polling the full list to find out is the only
-- alternative, and it cannot express deletion.
--
-- 2026-07-25 の運用決定: 評価（rating / user_tags / memo）のマスタは PromptBox。
-- したがってこのフィードは新規作成だけでなく更新・削除・復元まで全部流す。
--
-- The events are written by database triggers rather than by the application,
-- so imports, bulk operations, the PATCH endpoint and any future job all
-- produce them without having to remember to.
CREATE TABLE image_events (
    seq BIGSERIAL PRIMARY KEY,
    image_id UUID NOT NULL,
    -- created | updated | deleted | restored | purged
    kind VARCHAR(20) NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    payload JSONB NOT NULL DEFAULT '{}'::jsonb
);

COMMENT ON TABLE image_events IS '画像の変更フィード（下流の増分同期用・トリガーで自動記録）';
COMMENT ON COLUMN image_events.seq IS '単調増加カーソル。クライアントは ?since=<seq> で続きを取る';
COMMENT ON COLUMN image_events.kind IS 'created/updated/deleted(論理)/restored/purged(物理)';
COMMENT ON COLUMN image_events.payload IS 'updated では changed=[変更列]、purged では storage_path/file_hash';

-- No FK to images: a `purged` tombstone must outlive the row it describes.
CREATE INDEX idx_image_events_image_id ON image_events(image_id);

CREATE OR REPLACE FUNCTION record_image_event() RETURNS TRIGGER AS $$
DECLARE
    event_kind TEXT;
    changed TEXT[] := ARRAY[]::TEXT[];
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO image_events (image_id, kind) VALUES (NEW.id, 'created');
        RETURN NEW;
    END IF;

    IF TG_OP = 'DELETE' THEN
        -- Tombstone: the downstream copy is keyed by the original URL, so it
        -- needs the path to find what to remove.
        INSERT INTO image_events (image_id, kind, payload)
        VALUES (
            OLD.id,
            'purged',
            jsonb_build_object('storage_path', OLD.storage_path, 'file_hash', OLD.file_hash)
        );
        RETURN OLD;
    END IF;

    -- UPDATE: soft delete and restore are distinct kinds, everything else is
    -- an update carrying the list of fields that actually changed.
    IF OLD.deleted_at IS NULL AND NEW.deleted_at IS NOT NULL THEN
        event_kind := 'deleted';
    ELSIF OLD.deleted_at IS NOT NULL AND NEW.deleted_at IS NULL THEN
        event_kind := 'restored';
    ELSE
        event_kind := 'updated';

        IF OLD.rating IS DISTINCT FROM NEW.rating THEN
            changed := array_append(changed, 'rating');
        END IF;
        IF OLD.is_favorite IS DISTINCT FROM NEW.is_favorite THEN
            changed := array_append(changed, 'is_favorite');
        END IF;
        IF OLD.needs_improvement IS DISTINCT FROM NEW.needs_improvement THEN
            changed := array_append(changed, 'needs_improvement');
        END IF;
        IF OLD.user_tags IS DISTINCT FROM NEW.user_tags THEN
            changed := array_append(changed, 'user_tags');
        END IF;
        IF OLD.user_memo IS DISTINCT FROM NEW.user_memo THEN
            changed := array_append(changed, 'user_memo');
        END IF;

        IF array_length(changed, 1) IS NULL THEN
            -- Nothing a user edited. Either the row genuinely changed elsewhere
            -- (a re-parse rewriting model_name, prompts, …) or the UPDATE was a
            -- no-op that only bumped updated_at. Only the former is an event.
            IF (to_jsonb(OLD) - 'updated_at' - 'search_vector')
               IS DISTINCT FROM (to_jsonb(NEW) - 'updated_at' - 'search_vector') THEN
                changed := array_append(changed, 'metadata');
            ELSE
                RETURN NEW;
            END IF;
        END IF;
    END IF;

    INSERT INTO image_events (image_id, kind, payload)
    VALUES (NEW.id, event_kind, jsonb_build_object('changed', to_jsonb(changed)));
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_images_change_feed
    AFTER INSERT OR UPDATE OR DELETE ON images
    FOR EACH ROW
    EXECUTE FUNCTION record_image_event();

-- Existing libraries have no history. Seed one `created` event per live image
-- so a downstream that starts from since=0 sees the whole library rather than
-- only what changes from now on.
INSERT INTO image_events (image_id, kind, occurred_at, payload)
SELECT id, 'created', created_at, jsonb_build_object('backfilled', true)
FROM images
WHERE deleted_at IS NULL
ORDER BY created_at;
