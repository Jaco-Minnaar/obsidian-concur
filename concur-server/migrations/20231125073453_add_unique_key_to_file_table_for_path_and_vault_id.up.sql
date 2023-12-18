ALTER TABLE file ADD CONSTRAINT uc_path_vault_id UNIQUE (path, vault_id);
