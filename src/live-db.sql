DROP TABLE IF EXISTS positions;

CREATE TABLE positions (
    true_latitude DOUBLE NOT NULL,
    true_longitude DOUBLE NOT NULL,
    near_home BOOLEAN NOT NULL,
    latitude DOUBLE NOT NULL,
    longitude DOUBLE NOT NULL,
    local_timestamp INTEGER NOT NULL,
    gps_timestamp INTEGER NOT NULL,
    altitude DOUBLE NOT NULL,
    speed DOUBLE NOT NULL,
    hdop DOUBLE NOT NULL
);

CREATE INDEX positions_time ON positions(gps_timestamp);
