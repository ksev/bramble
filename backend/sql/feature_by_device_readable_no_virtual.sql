SELECT id, name, direction, kind, meta, automate FROM feature 
WHERE device = ? AND direction IN (1, 3)
ORDER BY name;