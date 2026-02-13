CREATE TABLE booking_legs (
    id UUID PRIMARY KEY,
    booking_id UUID NOT NULL REFERENCES bookings(id),
    flight_id UUID NOT NULL REFERENCES flights(id),
    seat_number VARCHAR(10),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Optional: Remove seat_number/flight_id from passengers/bookings if we want full normalization?
-- For now, let's keep passengers linked to booking, but we need to know WHICH flight they are on for WHICH seat.
-- Actually, a Passenger needs a seat per Flight.
-- New Table: passenger_seats(passenger_id, flight_id, seat_number)
-- OR, just keep it simple for MVP:
-- "passengers" table currently has "seat_number". This assumes 1 flight.
-- WE NEED TO CHANGE THIS.
-- A passenger has many seats (one per leg).

CREATE TABLE passenger_seats (
    passenger_id UUID NOT NULL REFERENCES passengers(id),
    flight_id UUID NOT NULL REFERENCES flights(id),
    seat_number VARCHAR(10) NOT NULL,
    PRIMARY KEY (passenger_id, flight_id)
);

-- We should drop seat_number from passengers table in a real migration.
-- ALTER TABLE passengers DROP COLUMN seat_number;
