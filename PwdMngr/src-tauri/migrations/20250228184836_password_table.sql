-- Create passwords table
CREATE TABLE IF NOT EXISTS passwords (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    website TEXT NOT NULL,
    website_url TEXT,
    encrypted_username TEXT NOT NULL,
    encrypted_password TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Add indexes for common queries
CREATE INDEX idx_passwords_user_id ON passwords(user_id);
CREATE INDEX idx_passwords_website ON passwords(website);