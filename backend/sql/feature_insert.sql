INSERT INTO feature (device, id, name, virtual, direction, kind, meta, automate) VALUES (?, ?, ?, ?, ?, ?, ?, ?) 
ON CONFLICT (device, id) DO 
    UPDATE SET direction=excluded.direction, 
               kind=excluded.kind, 
               meta=excluded.meta,
               virtual=excluded.virtual,
               automate=COALESCE(excluded.automate, automate) 