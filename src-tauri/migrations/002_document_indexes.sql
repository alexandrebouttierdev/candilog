CREATE INDEX IF NOT EXISTS idx_resume_versions_created_at
    ON resume_versions(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_cover_letters_created_at
    ON cover_letters(created_at DESC);
