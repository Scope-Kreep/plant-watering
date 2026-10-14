CREATE TABLE humidity_readings (
  id INTEGER PRIMARY KEY,
  timestamp INTEGER NOT NULL,
  raw_value INTEGER NOT NULL,
  percent REAL NOT NULL
)
