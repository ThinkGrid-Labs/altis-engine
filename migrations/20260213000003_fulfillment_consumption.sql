-- Add consumption tracking to fulfillment
ALTER TABLE fulfillment 
ADD COLUMN IF NOT EXISTS consumed_at TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS consumption_location VARCHAR(255);

-- Index for consumption queries
CREATE INDEX IF NOT EXISTS idx_fulfillment_consumed ON fulfillment(consumed_at) WHERE consumed_at IS NOT NULL;
