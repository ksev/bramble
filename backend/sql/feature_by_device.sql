SELECT id, name, virtual, direction, kind, meta, automate FROM feature WHERE device = ?
ORDER BY direction DESC, name;