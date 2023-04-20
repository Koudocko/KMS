CREATE TABLE groups (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  colour TEXT,
	vocab BOOLEAN NOT NULL,
  user_id INT NOT NULL,
  CONSTRAINT fk_user
    FOREIGN KEY(user_id)
     REFERENCES "users"(id)
);
