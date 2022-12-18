INSERT INTO device (id, name, type, parent,task_spec) VALUES (?,?,?,?,?) 
ON CONFLICT (id) DO UPDATE 
    SET name=excluded.name, 
        parent=excluded.parent,
        type=excluded.type,
        task_spec=excluded.task_spec