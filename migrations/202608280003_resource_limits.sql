-- Remove legacy soft-deleted links so deletion actually frees private writing
-- and quota capacity before the hard-delete behavior ships.
DELETE FROM feedback_loops WHERE deleted_at IS NOT NULL;

CREATE TRIGGER enforce_workspace_rubric_quota
BEFORE INSERT ON rubric_codes
WHEN (SELECT COUNT(*) FROM rubric_codes WHERE workspace_key = NEW.workspace_key) >= 100
BEGIN
    SELECT RAISE(ABORT, 'workspace rubric quota reached');
END;

CREATE TRIGGER enforce_workspace_loop_quota
BEFORE INSERT ON feedback_loops
WHEN (SELECT COUNT(*) FROM feedback_loops WHERE workspace_key = NEW.workspace_key) >= 500
BEGIN
    SELECT RAISE(ABORT, 'workspace loop quota reached');
END;

CREATE TRIGGER enforce_workspace_pack_quota
BEFORE INSERT ON rubric_packs
WHEN (SELECT COUNT(*) FROM rubric_packs WHERE workspace_key = NEW.workspace_key) >= 50
BEGIN
    SELECT RAISE(ABORT, 'workspace pack quota reached');
END;
