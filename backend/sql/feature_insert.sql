INSERT INTO feature (device, id, name, direction, kind, meta) VALUES (?, ?, ?, ?, ?, ?) 
ON CONFLICT (device, id) DO 
    UPDATE SET direction=excluded.direction, 
               kind=excluded.kind, 
               meta=excluded.meta