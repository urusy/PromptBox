-- C1: rebuild full-text search for AI prompts (docs/13 §3 観点③).
--
-- The old index/query pair used to_tsvector('english', …) + to_tsquery. Two
-- problems, both structural rather than tuning:
--
--   1. Prompts are comma-separated tag lists ("1girl, solo, (masterpiece:1.2)"),
--      not English prose. Stemming makes unrelated tags collide, and Japanese
--      prompts cannot be tokenised by an English dictionary at all.
--   2. The query text was fed to to_tsquery after a naive space -> " & "
--      replacement, so a prompt containing "!", "|" or "(" produced a tsquery
--      syntax error, i.e. an HTTP 500.
--
-- 'simple' does no stemming and drops no stop words, which is what a tag list
-- wants; the query side moves to websearch_to_tsquery, which never raises on
-- malformed input (and adds "phrases", -exclusion and `or` for free).
--
-- A stored generated column keeps the vector in sync with positive_prompt
-- automatically, including for the re-parse jobs planned in B4.
--
-- ⚠️ Adding a STORED generated column rewrites the table (ACCESS EXCLUSIVE).
-- On production (~42k rows) this takes seconds, but it is not concurrent —
-- see docs/runbooks/schema-baseline-migration.md.
ALTER TABLE images
    ADD COLUMN search_vector tsvector
    GENERATED ALWAYS AS (to_tsvector('simple', coalesce(positive_prompt, ''))) STORED;

COMMENT ON COLUMN images.search_vector IS
    'positive_prompt の全文検索ベクトル（simple 設定・自動更新）';

CREATE INDEX idx_images_search_vector ON images USING gin(search_vector);

-- Substring fallback for what tokenisation cannot reach: Japanese written
-- without separators collapses into a single token, so "少女" would never match
-- "美少女イラスト" through the tsvector alone. gin_trgm_ops makes the ILIKE arm
-- of the query index-backed instead of a sequential scan.
CREATE INDEX idx_images_positive_prompt_trgm ON images USING gin(positive_prompt gin_trgm_ops);

-- The english FTS indexes are now unused: the positive one is superseded above,
-- and nothing ever queried the negative one.
DROP INDEX IF EXISTS idx_images_positive_prompt_fts;
DROP INDEX IF EXISTS idx_images_negative_prompt_fts;
