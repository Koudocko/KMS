CREATE TABLE vocab (
  id SERIAL PRIMARY KEY,
  phrase TEXT NOT NULL,
  meaning TEXT NOT NULL,
  reading TEXT[] NOT NULL, 
  description TEXT,
  kanji_refs TEXT[] NOT NULL,
  user_id INT NOT NULL,
  CONSTRAINT fk_user
    FOREIGN KEY(user_id)
     REFERENCES "users"(id),
  group_id INT,
  CONSTRAINT fk_group
    FOREIGN KEY(group_id)
     REFERENCES "groups"(id)
);
