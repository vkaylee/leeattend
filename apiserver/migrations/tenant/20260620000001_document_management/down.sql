DROP TABLE IF EXISTS file_acknowledgements;
DROP TABLE IF EXISTS signature_requests;
ALTER TABLE workspace_files DROP CONSTRAINT IF EXISTS fk_workspace_files_folder_id;
DROP TABLE IF EXISTS document_folders;
