CREATE TABLE ais_position_reports (
    id SERIAL PRIMARY KEY,
    message_type INT,
    mmsi BIGINT NOT NULL,
    latitude DOUBLE PRECISION NOT NULL,
    longitude DOUBLE PRECISION NOT NULL,
    received_at TIMESTAMP DEFAULT NOW()
);


