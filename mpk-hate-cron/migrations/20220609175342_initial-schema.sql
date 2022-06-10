CREATE TABLE trips (
    trip_id varchar(20) CONSTRAINT firstkey PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actual_relative_time integer NOT NULL,
    actual_time time,
    direction varchar(100) NOT NULL,
    mixed_time varchar(15) NOT NULL,
    passageid varchar(20) NOT NULL,
    pattern_text varchar(20) NOT NULL,
    planned_time time NOT NULL,
    route_id varchar(20) NOT NULL,
    status varchar(10) NOT NULL,
    vehicle_id varchar(25) NOT NULL
);
