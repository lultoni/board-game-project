-- Design knowledge base schema.
-- One row per design artefact. Markdown bodies live in `body` columns.
-- Cross-refs live in `links`. No restated facts across tables.
--
-- Conventions:
--   - All IDs lowercase, hyphen-separated: "oq-42", "stack-m", "adr-004",
--     "session-26", "playtest-5".
--   - All dates ISO-8601 text: "2026-06-21".
--   - `body` is markdown. Renderers reassemble human-readable views.
--   - `status` is constrained per table via CHECK.
--   - `links` is the single generic cross-ref table.

PRAGMA foreign_keys = ON;


-- ============================================================
-- sessions
-- ============================================================
CREATE TABLE sessions (
  id           TEXT PRIMARY KEY,
  n            INTEGER NOT NULL UNIQUE,
  date         TEXT NOT NULL,
  title        TEXT,
  body         TEXT NOT NULL,
  created_at   TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at   TEXT NOT NULL DEFAULT (datetime('now'))
);


-- ============================================================
-- open_questions  (live + archived in one table; status separates them)
-- ============================================================
CREATE TABLE open_questions (
  id                TEXT PRIMARY KEY,
  title             TEXT NOT NULL,
  status            TEXT NOT NULL
                      CHECK (status IN (
                        'critical', 'high', 'medium', 'deferred', 'open',
                        'watch', 'resolved', 'closed', 'scrapped', 'parked',
                        'archived'
                      )),
  priority          INTEGER,
  affected_systems  TEXT,                    -- JSON array
  body              TEXT NOT NULL,
  resolved_in       TEXT,
  created_in        TEXT,
  created_at        TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at        TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (resolved_in) REFERENCES sessions(id),
  FOREIGN KEY (created_in)  REFERENCES sessions(id)
);


-- ============================================================
-- playtests
-- (Defined before `stacks` because stacks has FK to playtests.)
-- ============================================================
CREATE TABLE playtests (
  id                 TEXT PRIMARY KEY,
  n                  INTEGER NOT NULL UNIQUE,
  date               TEXT NOT NULL,
  players            TEXT NOT NULL,           -- JSON array
  stack_id           TEXT,                    -- nullable; FK added below via index only
  rounds             INTEGER,
  duration_min       INTEGER,
  outcome            TEXT,
  body               TEXT NOT NULL,
  raw_artefacts_path TEXT,
  created_at         TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at         TEXT NOT NULL DEFAULT (datetime('now'))
  -- NOTE: FK to stacks intentionally omitted at definition time to avoid
  -- circular constraint. Integrity check happens at migration time.
);


-- ============================================================
-- stacks  (test scenarios)
-- ============================================================
CREATE TABLE stacks (
  id            TEXT PRIMARY KEY,
  letter        TEXT NOT NULL UNIQUE,
  name          TEXT NOT NULL,
  status        TEXT NOT NULL
                  CHECK (status IN (
                    'active', 'queued', 'dormant', 'resolved',
                    'withdrawn', 'absorbed', 'archived'
                  )),
  hypothesis    TEXT,
  body          TEXT NOT NULL,
  playtested_in TEXT,
  created_in    TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (playtested_in) REFERENCES playtests(id),
  FOREIGN KEY (created_in)    REFERENCES sessions(id)
);


-- ============================================================
-- mechanics  (decision registry)
-- ============================================================
CREATE TABLE mechanics (
  id           TEXT PRIMARY KEY,
  name         TEXT NOT NULL,
  verdict      TEXT NOT NULL
                 CHECK (verdict IN (
                   'accepted', 'rejected', 'baseline', 'staged',
                   'superseded', 'withdrawn', 'pending'
                 )),
  source_oq    TEXT,
  body         TEXT NOT NULL,
  decided_in   TEXT,
  created_at   TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at   TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (source_oq)  REFERENCES open_questions(id),
  FOREIGN KEY (decided_in) REFERENCES sessions(id)
);


-- ============================================================
-- adrs  (architecture decision records)
-- ============================================================
CREATE TABLE adrs (
  id            TEXT PRIMARY KEY,
  n             INTEGER NOT NULL UNIQUE,
  title         TEXT NOT NULL,
  status        TEXT NOT NULL
                  CHECK (status IN (
                    'proposed', 'accepted', 'rejected',
                    'superseded', 'deprecated'
                  )),
  body          TEXT NOT NULL,
  decided_in    TEXT,
  superseded_by TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (decided_in)    REFERENCES sessions(id),
  FOREIGN KEY (superseded_by) REFERENCES adrs(id)
);


-- ============================================================
-- backpocket  (staged fixes, candidate skills, guardrails)
-- ============================================================
CREATE TABLE backpocket (
  id            TEXT PRIMARY KEY,
  name          TEXT NOT NULL,
  category      TEXT NOT NULL
                  CHECK (category IN (
                    'guardrail', 'staged-fix', 'skill-candidate',
                    'tooling', 'process', 'note', 'to-discuss'
                  )),
  status        TEXT NOT NULL
                  CHECK (status IN (
                    'active', 'parked', 'promoted', 'withdrawn'
                  )),
  fixes         TEXT,
  trigger_cond  TEXT,
  body          TEXT NOT NULL,
  promoted_to   TEXT,
  created_in    TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (created_in) REFERENCES sessions(id)
);


-- ============================================================
-- principles  (design principles + hard constraints + lenses)
-- ============================================================
CREATE TABLE principles (
  id             TEXT PRIMARY KEY,
  kind           TEXT NOT NULL
                   CHECK (kind IN ('principle', 'hard-constraint', 'lens', 'north-star')),
  n              INTEGER,
  title          TEXT NOT NULL,
  status         TEXT NOT NULL
                   CHECK (status IN ('active', 'superseded', 'retired')),
  body           TEXT NOT NULL,
  established_in TEXT,
  created_at     TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at     TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (established_in) REFERENCES sessions(id)
);


-- ============================================================
-- next_steps  (prioritised action items)
-- ============================================================
CREATE TABLE next_steps (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  priority      INTEGER NOT NULL,
  title         TEXT NOT NULL,
  status        TEXT NOT NULL
                  CHECK (status IN ('todo', 'in-progress', 'done', 'dropped')),
  body          TEXT,
  owner_oq      TEXT,
  owner_stack   TEXT,
  created_in    TEXT,
  completed_in  TEXT,
  created_at    TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at    TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (owner_oq)     REFERENCES open_questions(id),
  FOREIGN KEY (owner_stack)  REFERENCES stacks(id),
  FOREIGN KEY (created_in)   REFERENCES sessions(id),
  FOREIGN KEY (completed_in) REFERENCES sessions(id)
);


-- ============================================================
-- essays  (closed research / analyses — pure prose, no live status)
-- ============================================================
CREATE TABLE essays (
  id          TEXT PRIMARY KEY,
  title       TEXT NOT NULL,
  topic       TEXT,
  body        TEXT NOT NULL,
  date        TEXT,
  source_url  TEXT,
  created_at  TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);


-- ============================================================
-- design_docs  (living design directives — visual, frontend, onboarding...)
-- ============================================================
CREATE TABLE design_docs (
  id              TEXT PRIMARY KEY,
  title           TEXT NOT NULL,
  domain          TEXT NOT NULL,
  status          TEXT NOT NULL
                    CHECK (status IN ('active', 'superseded', 'retired')),
  body            TEXT NOT NULL,
  established_in  TEXT,
  created_at      TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
  FOREIGN KEY (established_in) REFERENCES sessions(id)
);


-- ============================================================
-- links  (generic cross-references between any two records)
-- ============================================================
CREATE TABLE links (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  from_id    TEXT NOT NULL,
  to_id      TEXT NOT NULL,
  relation   TEXT NOT NULL
               CHECK (relation IN (
                 'addresses',
                 'absorbed-into',
                 'supersedes',
                 'related-to',
                 'evidence-for',
                 'derived-from',
                 'connected-to',
                 'promoted-to',
                 'opened-by',
                 'resolved-by',
                 'blocks',
                 'parent-of'
               )),
  note       TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE (from_id, to_id, relation)
);

CREATE INDEX idx_links_from ON links(from_id);
CREATE INDEX idx_links_to   ON links(to_id);


-- ============================================================
-- updated_at triggers
-- ============================================================
CREATE TRIGGER trg_sessions_updated      AFTER UPDATE ON sessions
  BEGIN UPDATE sessions      SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_oqs_updated           AFTER UPDATE ON open_questions
  BEGIN UPDATE open_questions SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_stacks_updated        AFTER UPDATE ON stacks
  BEGIN UPDATE stacks        SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_mechanics_updated     AFTER UPDATE ON mechanics
  BEGIN UPDATE mechanics     SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_adrs_updated          AFTER UPDATE ON adrs
  BEGIN UPDATE adrs          SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_playtests_updated     AFTER UPDATE ON playtests
  BEGIN UPDATE playtests     SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_backpocket_updated    AFTER UPDATE ON backpocket
  BEGIN UPDATE backpocket    SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_principles_updated    AFTER UPDATE ON principles
  BEGIN UPDATE principles    SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_next_steps_updated    AFTER UPDATE ON next_steps
  BEGIN UPDATE next_steps    SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_essays_updated        AFTER UPDATE ON essays
  BEGIN UPDATE essays        SET updated_at = datetime('now') WHERE id = OLD.id; END;
CREATE TRIGGER trg_design_docs_updated   AFTER UPDATE ON design_docs
  BEGIN UPDATE design_docs   SET updated_at = datetime('now') WHERE id = OLD.id; END;
