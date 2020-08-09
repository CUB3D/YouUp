-- This file should undo anything in `up.sql`
ALTER TABLE incidents ADD COLUMN status Varchar(32) NOT NULL;
ALTER TABLE incidents ADD COLUMN message Varchar(256) NOT NULL;
