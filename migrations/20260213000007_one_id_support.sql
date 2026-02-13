-- Add customer_did to orders table for IATA One Identity support
ALTER TABLE orders ADD COLUMN customer_did TEXT;
