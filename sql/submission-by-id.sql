SELECT
    id,
    teamName AS team_name,
    milestone,
    description,
    attachmentLink as attachment,
    approved,
    creationTime as creation_time
FROM submissions
WHERe id = $1
ORDER BY creationTime DESC;
