-- Add Expiration to Orders (Order Hold Time)
-- This allows PROPOSED orders to have a limited lifetime before inventory is released.

ALTER TABLE orders ADD COLUMN IF NOT EXISTS expires_at TIMESTAMPTZ;

-- Index for efficient cleanup of expired orders
CREATE INDEX IF NOT EXISTS idx_orders_expires ON orders(expires_at) WHERE status = 'PROPOSED';
