DROP TABLE IF EXISTS positions;
DROP TABLE IF EXISTS clusters;

CREATE TABLE positions (
       true_latitude DOUBLE NOT NULL,
       true_longitude DOUBLE NOT NULL,
       near_home BOOLEAN NOT NULL,
       latitude DOUBLE NOT NULL,
       longitude DOUBLE NOT NULL,
       camera_timestamp INTEGER NOT NULL,
       gps_timestamp INTEGER NOT NULL,
       filename TEXT,
       cluster_id INTEGER
);

CREATE TABLE clusters (
       latitude DOUBLE NOT NULL,
       longitude DOUBLE NOT NULL,
       camera_timestamp INTEGER NOT NULL,
       gps_timestamp INTEGER NOT NULL,
       num_points INTEGER NOT NULL
);

CREATE INDEX positions_spacetime ON positions (latitude, longitude, gps_timestamp);
CREATE INDEX positions_filename ON positions (filename);
CREATE INDEX positions_spacetimefile ON positions (latitude, longitude, gps_timestamp, filename);
CREATE INDEX positions_cluster_id ON positions (cluster_id);
