SELECT id, name, virtual, direction, kind, meta, automate FROM feature 
WHERE device = ? AND direction IN (1, 3) AND virtual = false
ORDER BY name;