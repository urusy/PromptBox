-- B5: lightweight background job infrastructure (docs/13 §3 観点②).
--
-- Re-parsing, pHash backfill, derived thumbnails, orphan GC, trash purge,
-- statistics refresh and prompt tokenisation are all the same shape: a long
-- running task started over HTTP whose progress must be pollable. This table
-- is that shape, so those features become job kinds rather than seven bespoke
-- endpoints.
--
-- Deliberately NOT a queueing framework: PromptBox is a single instance, so a
-- table plus a Tokio task with a semaphore of 1 is the whole mechanism.
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    kind VARCHAR(50) NOT NULL,
    -- queued | running | succeeded | failed | cancelled | interrupted
    status VARCHAR(20) NOT NULL,
    params JSONB NOT NULL DEFAULT '{}'::jsonb,
    progress_current BIGINT NOT NULL DEFAULT 0,
    -- NULL until the job knows how much work there is.
    progress_total BIGINT,
    result JSONB,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT jobs_progress_non_negative CHECK (progress_current >= 0)
);

COMMENT ON TABLE jobs IS 'バックグラウンドジョブ（再解析・GC・集計などの実行状態）';
COMMENT ON COLUMN jobs.status IS 'queued/running/succeeded/failed/cancelled/interrupted';
COMMENT ON COLUMN jobs.error IS '失敗理由（status=failed のとき）';

-- Polling a single job by id uses the primary key; these two cover the list
-- view and the startup sweep for jobs left running by a crash.
CREATE INDEX idx_jobs_created_at ON jobs(created_at DESC);
CREATE INDEX idx_jobs_status ON jobs(status);

CREATE TRIGGER trigger_jobs_updated_at
    BEFORE UPDATE ON jobs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
