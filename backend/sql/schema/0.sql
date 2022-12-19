CREATE TABLE "device" (
	"id"	TEXT NOT NULL,
	"name"	TEXT NOT NULL,
	"type"	TEXT NOT NULL,
	"parent"	TEXT,
	"task_spec"	TEXT NOT NULL,
	FOREIGN KEY("parent") REFERENCES "device"("id") ON DELETE CASCADE,
	PRIMARY KEY("id")
);

CREATE TABLE "feature" (
	"device"	TEXT NOT NULL,
	"id"	TEXT NOT NULL,
	"name"	TEXT NOT NULL,
	"direction"	INTEGER NOT NULL,
	"kind"	INTEGER NOT NULL,
	"meta"	TEXT NOT NULL,
	FOREIGN KEY("device") REFERENCES "device"("id") ON DELETE CASCADE,
	PRIMARY KEY("device","id")
);
