CREATE TABLE IF NOT EXISTS countdowns (
    id INTEGER PRIMARY KEY,
    label TEXT NOT NULL,
    date TEXT NOT NULL,
    repeat TEXT NOT NULL,
    lunar_leap INTEGER NOT NULL
);
