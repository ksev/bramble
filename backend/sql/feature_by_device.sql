SELECT id, name, direction, kind, meta, automate FROM feature WHERE device = ?
ORDER BY direction DESC, name;