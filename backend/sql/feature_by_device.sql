SELECT id, name, direction, kind, meta FROM feature WHERE device = ?
ORDER BY direction DESC, name;