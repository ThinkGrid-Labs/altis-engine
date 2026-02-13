-- Fix schema mismatches for flight_repo.rs
ALTER TABLE flights ADD COLUMN IF NOT EXISTS base_price_amount INT NOT NULL DEFAULT 10000;
ALTER TABLE flights ADD COLUMN IF NOT EXISTS base_price_currency CHAR(3) NOT NULL REFERENCES currencies(code) DEFAULT 'USD';
ALTER TABLE aircraft_configs ADD COLUMN IF NOT EXISTS capacity INT NOT NULL DEFAULT 150;
