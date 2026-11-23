CREATE TABLE IF NOT EXISTS timers (
    id INTEGER PRIMARY KEY,
    label TEXT NOT NULL,
    end_seconds INTEGER NOT NULL,
    paused_seconds INTEGER
);
