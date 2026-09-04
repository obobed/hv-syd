CREATE TABLE IF NOT EXISTS attendees (
    userId      TEXT  NOT NULL UNIQUE PRIMARY KEY,
    teamId      TEXT  UNIQUE,
    firstName   TEXT  NOT NULL,
    lastName    TEXT  NOT NULL,
    pwHash      TEXT  NOT NULL,

    FOREIGN KEY(teamId) REFERENCES teams(teamId)
);

CREATE TABLE IF NOT EXISTS teams (
    teamId      TEXT  NOT NULL UNIQUE PRIMARY KEY,
    teamName    TEXT  NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS submissions (
    subId           TEXT     NOT NULL UNIQUE PRIMARY KEY,
    teamId          TEXT     NOT NULL UNIQUE,
    milestone       TEXT     NOT NULL,
    description     TEXT,
    attachmentLink  TEXT,
    approved        BOOLEAN  NOT NULL DEFAULT FALSE,

    FOREIGN KEY(teamId) REFERENCES teams(teamId)
);

CREATE INDEX IF NOT EXISTS ix_attendees ON attendees (
    userId,
    teamId
);

CREATE INDEX IF NOT EXISTS ix_teams on teams (
    teamId,
    teamName
);

CREATE INDEX IF NOT EXISTS ix_submissions ON submissions (
    subId,
    teamId,
    milestone
);
