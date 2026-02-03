-- Add migration script here

CREATE TABLE IF NOT EXISTS messages (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,               -- Foreign Key to app_users
    content TEXT NOT NULL,              -- The actual message
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexing is CRITICAL for history speed
    INDEX (created_at), 
    FOREIGN KEY (user_id) REFERENCES app_users(id) ON DELETE CASCADE
);