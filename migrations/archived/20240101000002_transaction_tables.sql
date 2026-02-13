-- Transaction Tables

-- Flights (Instances of routes)
CREATE TABLE flights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    route_id UUID NOT NULL REFERENCES routes(id),
    aircraft_config_id UUID NOT NULL REFERENCES aircraft_configs(id),
    flight_number VARCHAR(20) NOT NULL,
    scheduled_departure TIMESTAMPTZ NOT NULL,
    scheduled_arrival TIMESTAMPTZ NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'SCHEDULED', -- SCHEDULED, CANCELLED, DELAYED, DEPARTED
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Bookings (PNR)
CREATE TABLE bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    flight_id UUID NOT NULL REFERENCES flights(id),
    user_email VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'PENDING', -- PENDING, CONFIRMED, CANCELLED, EXPIRED
    total_price_amount INT NOT NULL,
    total_price_currency CHAR(3) NOT NULL REFERENCES currencies(code),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Passengers
CREATE TABLE passengers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    booking_id UUID NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    date_of_birth DATE,
    seat_number VARCHAR(10), -- Assigned seat, nullable until check-in/selection
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_flights_route_date ON flights(route_id, scheduled_departure);
CREATE INDEX idx_bookings_user ON bookings(user_email);
CREATE INDEX idx_bookings_flight ON bookings(flight_id);
