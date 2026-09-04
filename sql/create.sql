CREATE TABLE IF NOT EXISTS attendees (
    userId      TEXT  NOT NULL UNIQUE PRIMARY KEY,
    teamName    TEXT,
    firstName   TEXT  NOT NULL,
    lastName    TEXT  NOT NULL,
    pwHash      TEXT  NOT NULL,

    FOREIGN KEY(teamName) REFERENCES teams(teamName)
);

CREATE TABLE IF NOT EXISTS teams (
    teamName    TEXT  NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS submissions (
    subId           TEXT     NOT NULL UNIQUE PRIMARY KEY,
    teamName        TEXT     NOT NULL,
    milestone       TEXT     NOT NULL,
    description     TEXT,
    attachmentLink  TEXT,
    approved        BOOLEAN  NOT NULL DEFAULT FALSE,

    FOREIGN KEY(teamName) REFERENCES teams(teamName)
);

CREATE INDEX IF NOT EXISTS ix_attendees ON attendees (
    userId,
    teamName
);

CREATE INDEX IF NOT EXISTS ix_teams on teams (
    teamName
);

CREATE INDEX IF NOT EXISTS ix_submissions ON submissions (
    subId,
    teamName,
    milestone
);
