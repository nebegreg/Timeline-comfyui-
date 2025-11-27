-- Plugin Marketplace Database Schema
-- SQLite database for plugin catalog and ratings

-- Plugins table
CREATE TABLE IF NOT EXISTS plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    long_description TEXT,
    version TEXT NOT NULL,
    author TEXT NOT NULL,
    author_email TEXT,
    plugin_type TEXT NOT NULL CHECK (plugin_type IN ('wasm', 'python')),
    category TEXT NOT NULL,
    tags TEXT NOT NULL,
    download_url TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    file_hash TEXT NOT NULL,
    license TEXT NOT NULL,
    homepage TEXT,
    screenshots TEXT,
    downloads INTEGER DEFAULT 0,
    rating REAL DEFAULT 0.0,
    rating_count INTEGER DEFAULT 0,
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Ratings table
CREATE TABLE IF NOT EXISTS ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plugin_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (plugin_id) REFERENCES plugins(id) ON DELETE CASCADE,
    UNIQUE(plugin_id, user_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_plugins_category ON plugins(category);
CREATE INDEX IF NOT EXISTS idx_plugins_type ON plugins(plugin_type);
CREATE INDEX IF NOT EXISTS idx_plugins_downloads ON plugins(downloads DESC);
CREATE INDEX IF NOT EXISTS idx_plugins_rating ON plugins(rating DESC);
CREATE INDEX IF NOT EXISTS idx_plugins_created ON plugins(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ratings_plugin ON ratings(plugin_id);

-- Trigger to update plugin rating when a new rating is added
CREATE TRIGGER IF NOT EXISTS update_plugin_rating
AFTER INSERT ON ratings
BEGIN
    UPDATE plugins
    SET rating = (
        SELECT AVG(rating) FROM ratings WHERE plugin_id = NEW.plugin_id
    ),
    rating_count = (
        SELECT COUNT(*) FROM ratings WHERE plugin_id = NEW.plugin_id
    )
    WHERE id = NEW.plugin_id;
END;

-- Trigger to update plugin rating when a rating is updated
CREATE TRIGGER IF NOT EXISTS update_plugin_rating_on_update
AFTER UPDATE ON ratings
BEGIN
    UPDATE plugins
    SET rating = (
        SELECT AVG(rating) FROM ratings WHERE plugin_id = NEW.plugin_id
    ),
    rating_count = (
        SELECT COUNT(*) FROM ratings WHERE plugin_id = NEW.plugin_id
    )
    WHERE id = NEW.plugin_id;
END;

-- Trigger to update plugin rating when a rating is deleted
CREATE TRIGGER IF NOT EXISTS update_plugin_rating_on_delete
AFTER DELETE ON ratings
BEGIN
    UPDATE plugins
    SET rating = COALESCE((
        SELECT AVG(rating) FROM ratings WHERE plugin_id = OLD.plugin_id
    ), 0.0),
    rating_count = (
        SELECT COUNT(*) FROM ratings WHERE plugin_id = OLD.plugin_id
    )
    WHERE id = OLD.plugin_id;
END;
