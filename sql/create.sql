CREATE TABLE IF NOT EXISTS attendees (
    id          TEXT  NOT NULL UNIQUE PRIMARY KEY,
    teamName    TEXT,
    firstName   TEXT  NOT NULL,
    lastName    TEXT  NOT NULL,
    pwHash      TEXT  NOT NULL,

    FOREIGN KEY(teamName)
    REFERENCES teams(teamName)
    ON UPDATE CASCADE
    ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS teams (
    teamName    TEXT  NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS submissions (
    id              TEXT     NOT NULL UNIQUE PRIMARY KEY,
    teamName        TEXT     NOT NULL,
    milestone       TEXT     NOT NULL,
    description     TEXT,
    attachmentLink  TEXT,
    approved        BOOLEAN  NOT NULL DEFAULT FALSE,

    FOREIGN KEY(teamName)
    REFERENCES teams(teamName)
    ON UPDATE CASCADE
    ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS ix_attendees ON attendees (
    id,
    teamName
);

CREATE INDEX IF NOT EXISTS ix_teams on teams (
    teamName
);

CREATE INDEX IF NOT EXISTS ix_submissions ON submissions (
    id,
    teamName,
    milestone
);
