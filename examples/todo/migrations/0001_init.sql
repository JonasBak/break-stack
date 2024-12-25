CREATE TABLE IF NOT EXISTS todo_items
(
    id           INTEGER PRIMARY KEY NOT NULL,
    description  TEXT                NOT NULL,
    done         BOOLEAN             NOT NULL
);
