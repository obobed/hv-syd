SELECT
    json_group_array(
        json_object(
            'id', id,
            'first_name', firstName,
            'last_name', lastName
        )
    ) as "attendees: _",
    teams.teamName as team_name
FROM teams, attendees
WHERE
    teams.teamName = attendees.teamName AND
    teams.teamName = $1
GROUP BY teams.teamName;
