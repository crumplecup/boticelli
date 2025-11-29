-- Create content table for actor system
CREATE TABLE content (
    id SERIAL PRIMARY KEY,
    content_type VARCHAR(50) NOT NULL,
    text_content TEXT,
    media_urls TEXT[],
    media_types VARCHAR(20)[],
    source VARCHAR(255),
    priority INTEGER DEFAULT 0,
    tags TEXT[],
    approved_at TIMESTAMP,
    approved_by VARCHAR(100),
    scheduled_for TIMESTAMP,
    expires_at TIMESTAMP,
    post_count INTEGER DEFAULT 0,
    last_posted_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    metadata JSONB
);

CREATE INDEX idx_content_approved ON content(approved_at);
CREATE INDEX idx_content_scheduled ON content(scheduled_for);
CREATE INDEX idx_content_priority ON content(priority DESC);
CREATE INDEX idx_content_tags ON content USING GIN(tags);

-- Create post_history table to track all posts made by actors
CREATE TABLE post_history (
    id SERIAL PRIMARY KEY,
    content_id INTEGER REFERENCES content(id),
    actor_name VARCHAR(100) NOT NULL,
    platform VARCHAR(50) NOT NULL,
    channel_id VARCHAR(100),
    post_id VARCHAR(255),
    posted_at TIMESTAMP DEFAULT NOW() NOT NULL,
    engagement_count INTEGER DEFAULT 0,
    metadata JSONB
);

CREATE INDEX idx_post_history_posted ON post_history(posted_at DESC);
CREATE INDEX idx_post_history_content ON post_history(content_id);
CREATE INDEX idx_post_history_actor ON post_history(actor_name, posted_at);

-- Create actor_preferences table for actor-specific configuration
CREATE TABLE actor_preferences (
    id SERIAL PRIMARY KEY,
    actor_name VARCHAR(100) UNIQUE NOT NULL,
    min_post_interval_minutes INTEGER DEFAULT 60,
    max_posts_per_day INTEGER DEFAULT 10,
    preferred_tags TEXT[],
    excluded_tags TEXT[],
    time_window_start TIME,
    time_window_end TIME,
    timezone VARCHAR(50) DEFAULT 'UTC',
    randomize_schedule BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP DEFAULT NOW() NOT NULL
);
