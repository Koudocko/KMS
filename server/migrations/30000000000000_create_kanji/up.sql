CREATE TABLE kanji (
  id SERIAL PRIMARY KEY,
  symbol TEXT NOT NULL,
  meaning TEXT NOT NULL,
  onyomi TEXT[] NOT NULL, 
  kunyomi TEXT[] NOT NULL, 
  description TEXT,
  vocab_refs TEXT[] NOT NULL,
  user_id INT NOT NULL,
  CONSTRAINT fk_user
    FOREIGN KEY(user_id)
     REFERENCES "users"(id),
  group_id INT,
  CONSTRAINT fk_group
    FOREIGN KEY(group_id)
     REFERENCES "groups"(id)
);
