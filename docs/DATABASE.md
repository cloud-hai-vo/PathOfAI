# Path of AI — Database Schema (SQLite)

Stored in `PathOfAI_Data/path-of-ai.db`. Single SQLite file, portable.

---

## Tables

### builds
```sql
CREATE TABLE builds (
  id          TEXT PRIMARY KEY,           -- UUID
  file_path   TEXT NOT NULL,              -- absolute path to PoB XML
  name        TEXT,                       -- user-given name or auto-detected
  class_name  TEXT NOT NULL,              -- "Templar"
  ascendancy  TEXT NOT NULL,              -- "Inquisitor"
  level       INTEGER NOT NULL,
  main_skill  TEXT,                       -- "RighteousFire"
  archetype   TEXT,                       -- "fire_dot"
  total_dps   REAL,
  total_life  INTEGER,
  score       INTEGER,                    -- 0-100 overall
  last_analyzed TEXT NOT NULL,            -- ISO8601 timestamp
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);
```

### build_snapshots (undo/redo history)
```sql
CREATE TABLE build_snapshots (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  build_id    TEXT NOT NULL REFERENCES builds(id),
  xml_content BLOB NOT NULL,              -- compressed PoB XML
  description TEXT,                       -- "Before Ring 2 upgrade"
  created_at  TEXT NOT NULL,
  FOREIGN KEY (build_id) REFERENCES builds(id)
);
-- Keep last 50 per build
CREATE INDEX idx_snapshots_build ON build_snapshots(build_id, created_at DESC);
```

### price_cache
```sql
CREATE TABLE price_cache (
  item_key    TEXT PRIMARY KEY,           -- "unique:Aegis Aurora" or "base:Opal Ring"
  price_chaos REAL NOT NULL,
  price_divine REAL NOT NULL,
  league      TEXT NOT NULL,
  fetched_at  TEXT NOT NULL,              -- ISO8601
  source      TEXT DEFAULT 'poe.ninja'
);
CREATE INDEX idx_price_fetched ON price_cache(fetched_at);
```

### price_history (for trends)
```sql
CREATE TABLE price_history (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  item_key    TEXT NOT NULL,
  price_divine REAL NOT NULL,
  recorded_at TEXT NOT NULL,
  league      TEXT NOT NULL
);
CREATE INDEX idx_price_history ON price_history(item_key, recorded_at);
```

### wealth_snapshots
```sql
CREATE TABLE wealth_snapshots (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  total_divine REAL NOT NULL,
  currency_breakdown TEXT,                -- JSON: { divine: 12, chaos: 340, ... }
  stash_value REAL,
  gear_value  REAL,
  recorded_at TEXT NOT NULL
);
```

### settings
```sql
CREATE TABLE settings (
  key         TEXT PRIMARY KEY,
  value       TEXT NOT NULL               -- JSON-encoded value
);
-- Example rows:
-- ('pob_path', '"/path/to/PoB"')
-- ('ai_provider', '"seer"')
-- ('game_version', '"poe1"')
-- ('theme', '"dark"')
```

### alerts
```sql
CREATE TABLE alerts (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  alert_type  TEXT NOT NULL,              -- 'price_drop', 'snipe', 'currency_rate'
  alert_name  TEXT,                       -- user-friendly label
  item_key    TEXT,
  threshold   REAL,
  comparison  TEXT DEFAULT 'below',       -- 'below', 'above', 'change_percent'
  notify_method TEXT DEFAULT 'popup',     -- 'popup', 'sound', 'silent'
  active      INTEGER DEFAULT 1,
  created_at  TEXT NOT NULL,
  last_triggered TEXT
);
```

### map_runs (Mapwatch-style tracking)
```sql
CREATE TABLE map_runs (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  build_id    TEXT REFERENCES builds(id),
  map_name    TEXT NOT NULL,
  map_tier    INTEGER NOT NULL,
  clear_time_ms INTEGER,                  -- milliseconds to clear
  deaths      INTEGER DEFAULT 0,
  xp_gained   INTEGER,
  currency_dropped TEXT,                  -- JSON: { "chaos": 5, "divine": 0 }
  map_mods    TEXT,                       -- JSON array of mod names
  started_at  TEXT NOT NULL,
  finished_at TEXT
);
CREATE INDEX idx_map_runs_build ON map_runs(build_id, started_at DESC);
```

### session_stats
```sql
CREATE TABLE session_stats (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  build_id    TEXT REFERENCES builds(id),
  maps_run    INTEGER DEFAULT 0,
  total_deaths INTEGER DEFAULT 0,
  total_xp    INTEGER DEFAULT 0,
  currency_earned TEXT,                   -- JSON: total currency this session
  avg_clear_ms INTEGER,
  best_clear_ms INTEGER,
  session_start TEXT NOT NULL,
  session_end TEXT
);
```

### div_card_progress
```sql
CREATE TABLE div_card_progress (
  card_name   TEXT PRIMARY KEY,
  owned       INTEGER DEFAULT 0,
  required    INTEGER NOT NULL,           -- total needed for turn-in
  reward      TEXT,                       -- what the card gives
  drop_locations TEXT,                    -- JSON array of map names
  updated_at  TEXT NOT NULL
);
```

### character_data (from PoE OAuth)
```sql
CREATE TABLE character_data (
  name        TEXT PRIMARY KEY,
  class_name  TEXT NOT NULL,
  level       INTEGER NOT NULL,
  league      TEXT NOT NULL,
  last_synced TEXT NOT NULL
);
```

### league_currency (per-league tracking)
```sql
CREATE TABLE league_currency (
  currency_name TEXT NOT NULL,
  count         INTEGER DEFAULT 0,
  league        TEXT NOT NULL,
  updated_at    TEXT NOT NULL,
  PRIMARY KEY (currency_name, league)
);
-- Tracks league-specific currencies (Djinn Coins, etc.)
-- Cleared when league ends
```

### schema_version
```sql
CREATE TABLE schema_version (
  version     INTEGER PRIMARY KEY,
  applied_at  TEXT NOT NULL,
  description TEXT
);
-- Current: INSERT INTO schema_version VALUES (1, datetime('now'), 'Initial schema');
-- See AUTO-UPDATE-SYSTEM.md §10 for migration strategy
```

---

## Data Retention Policy

```
build_snapshots: max 50 per build, auto-delete oldest
price_cache:     TTL 5 minutes (auto-evict stale entries)
price_history:   keep 90 days, auto-delete older
map_runs:        keep current league only (delete on league end)
session_stats:   keep current league only
wealth_snapshots: keep 90 days
backups (files):  keep 30 days, max 50 per build
```

## Migration Strategy

```sql
-- On app startup, check schema_version
-- If version < current → run migration scripts in order

-- Migration v1 → v2 example:
ALTER TABLE alerts ADD COLUMN alert_name TEXT;
ALTER TABLE alerts ADD COLUMN comparison TEXT DEFAULT 'below';
ALTER TABLE alerts ADD COLUMN notify_method TEXT DEFAULT 'popup';
INSERT INTO schema_version VALUES (2, datetime('now'), 'Added alert fields');
```
