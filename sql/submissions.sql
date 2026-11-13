SELECT
    id,
    teamName AS team_name,
    milestone,
    description,
    attachmentLink as attachment,
    approved,
    rejected,
    creationTime as creation_time
FROM submissions
ORDER BY creationTime DESC;
