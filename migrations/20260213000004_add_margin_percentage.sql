-- Add margin_percentage to products table
ALTER TABLE products ADD COLUMN margin_percentage DOUBLE PRECISION DEFAULT 0.1500;
