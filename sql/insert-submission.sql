INSERT INTO submissions (
    id,
    teamName,
    milestone,
    description,
    attachmentLink,
    approved,
    creationTime
) VALUES ( $1, $2, $3, $4, $5, $6, $7 )
