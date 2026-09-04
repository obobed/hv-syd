CREATE TABLE IF NOT EXISTS attendees (
    userId      INT   NOT NULL UNIQUE PRIMARY KEY,
    teamId      INT   UNIQUE,
    firstName   TEXT  NOT NULL,
    lastName    TEXT  NOT NULL,
    pwHash      TEXT  NOT NULL,

    FOREIGN KEY(teamId) REFERENCES teams(teamId)
);

CREATE TABLE IF NOT EXISTS teams (
    teamId      INT   NOT NULL UNIQUE PRIMARY KEY,
    teamName    TEXT  NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS submissions (
    subId           INT      NOT NULL UNIQUE PRIMARY KEY,
    teamId          INT      NOT NULL UNIQUE,
    milestone       TEXT     NOT NULL,
    description     TEXT,
    attachmentLink  TEXT,
    approved        BOOLEAN  NOT NULL DEFAULT FALSE,

    FOREIGN KEY(teamId) REFERENCES teams(teamId)
);
