INSERT INTO feature (device, id, name, direction, kind, meta, automate) VALUES (?, ?, ?, ?, ?, ?, ?) 
ON CONFLICT (device, id) DO 
    UPDATE SET direction=excluded.direction, 
               kind=excluded.kind, 
               meta=excluded.meta,
               automate=COALESCE(excluded.automate, automate) 