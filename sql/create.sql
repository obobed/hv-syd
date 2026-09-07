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
    milestone       INTEGER  NOT NULL,
    description     TEXT,
    attachmentLink  TEXT,
    approved        BOOLEAN  NOT NULL DEFAULT FALSE,

    FOREIGN KEY(teamName)
    REFERENCES teams(teamName)
    ON UPDATE CASCADE
    ON DELETE CASCADE

    FOREIGN KEY(milestone)
    REFERENCES milestones(id)
    ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS milestones (
    id            INTEGER  NOT NULL UNIQUE PRIMARY KEY,
    description   TEXT     NOT NULL,
    requirements  TEXT     NOT NULL, -- Vec<String>
    optional      BOOLEAN  NOT NULL
);

CREATE TABLE IF NOT EXISTS prizes (
    prizeName         TEXT  NOT NULL UNIQUE PRIMARY KEY,
    description       TEXT  NOT NULL,
    image             TEXT,
    participantNotes  TEXT, -- Option<Vec<String>>
    stock             INTEGER,
    milestone         INTEGER NOT NULL,

    FOREIGN KEY(milestone)
    REFERENCES milestones(id)
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

CREATE INDEX IF NOT EXISTS ix_milestones ON milestones (
    id
);

CREATE INDEX IF NOT EXISTS ix_prizes ON prizes (
    prizeName,
    milestone
);
