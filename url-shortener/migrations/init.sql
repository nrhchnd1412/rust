CREATE TABLE IF NOT EXISTS urls (
                                    id BIGSERIAL PRIMARY KEY,
                                    short_code VARCHAR(12) NOT NULL UNIQUE,
    original_url TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    expires_at TIMESTAMP WITH TIME ZONE NULL,
                             is_deleted BOOLEAN DEFAULT FALSE,
                             clicks BIGINT DEFAULT 0
                             );

CREATE TABLE IF NOT EXISTS clicks (
                                      id BIGSERIAL PRIMARY KEY,
                                      url_id BIGINT NOT NULL REFERENCES urls(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT now(),
    ip INET NULL,
    user_agent TEXT NULL,
    referer TEXT NULL
    );
