-- Reference Tables

-- Currencies
CREATE TABLE currencies (
    code CHAR(3) PRIMARY KEY, -- "USD", "EUR"
    name VARCHAR(255) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    precision INT NOT NULL DEFAULT 2
);

-- Airports
CREATE TABLE airports (
    iata_code CHAR(3) PRIMARY KEY, -- "LHR", "JFK"
    city VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    timezone VARCHAR(255) NOT NULL
);

-- Aircraft Configurations (LOPA)
CREATE TABLE aircraft_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    seating_layout JSONB NOT NULL, -- Flexible layout definition
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Routes
CREATE TABLE routes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    origin_iata CHAR(3) NOT NULL REFERENCES airports(iata_code),
    destination_iata CHAR(3) NOT NULL REFERENCES airports(iata_code),
    distance_miles INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(origin_iata, destination_iata)
);
