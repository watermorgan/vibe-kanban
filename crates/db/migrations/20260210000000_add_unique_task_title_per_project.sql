-- Add unique constraint on tasks(project_id, title) to prevent duplicate task titles within a project
-- This is enforced at the application level via COLLATE NOCASE for case-insensitive comparison

-- Create a unique index on (project_id, title) with case-insensitive collation
-- SQLite handles case-insensitive uniqueness via COLLATE NOCASE
CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_project_id_title
ON tasks(project_id, title COLLATE NOCASE);
