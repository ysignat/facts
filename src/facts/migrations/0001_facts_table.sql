CREATE TABLE IF NOT EXISTS facts (
  id serial PRIMARY KEY,
  title varchar(64) NOT NULL CHECK (title <> ''),
  body varchar(2048) NOT NULL CHECK (body <> '')
)