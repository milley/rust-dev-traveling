CREATE SCHEMA rsvp;

CREATE TABLE todos (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    completed bool NOT NULL default false
);