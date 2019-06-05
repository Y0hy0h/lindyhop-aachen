CREATE TABLE events (
    id BINARY(128) PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    teaser VARCHAR NOT NULL,
    description VARCHAR NOT NULL
);
CREATE TABLE locations (
    id BINARY(128) PRIMARY KEY NOT NULL,
    name VARCHAR NOT NULL,
    address VARCHAR NOT NULL
);
CREATE TABLE occurrences (
    id BINARY(128) PRIMARY KEY NOT NULL,
    start TIMESTAMP NOT NULL,
    end TIMESTAMP NOT NULL,
    event_id BINARY(128) NOT NULL,
    location_id BINARY(128) NOT NULL,
    FOREIGN KEY (event_id) REFERENCES events(id),
    FOREIGN KEY (location_id) REFERENCES locations(id)
);
