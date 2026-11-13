SELECT
    id,
    name,
    milestones.description,
    json(requirements) as "requirements: _",
    json_group_array(prizeName) as "prizes: _",
    optional
FROM milestones, prizes
WHERE milestones.id = prizes.milestone
GROUP BY milestones.id;
-- (
--     SELECT
--         milestones.id,
--         milestones.name,
--         milestones.description,
--         milestones.requirements,
--         milestones.optional,
--         json_group_array(prizes.prizeName) AS prize_array
--     FROM milestones
--     LEFT JOIN prizes ON milestones.id = prizes.milestone
--     GROUP BY milestones.id
-- );
