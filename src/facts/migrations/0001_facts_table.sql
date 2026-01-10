CREATE TABLE IF NOT EXISTS facts (
  id integer PRIMARY KEY,
  title varchar(64) NOT NULL CHECK (title <> ''),
  body varchar(2048) NOT NULL CHECK (body <> '')
)