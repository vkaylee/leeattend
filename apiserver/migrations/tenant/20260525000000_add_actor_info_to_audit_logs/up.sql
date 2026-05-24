ALTER TABLE audit_logs 
    ADD COLUMN actor_type VARCHAR(50),
    ADD COLUMN actor_name VARCHAR(255),
    ADD COLUMN actor_email VARCHAR(255);
