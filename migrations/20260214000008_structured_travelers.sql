-- Add Structured Passenger and Contact Support
-- Aligned with IATA NDC and ONE Order standards

-- 1. Create TRAVELERS table
CREATE TABLE IF NOT EXISTS travelers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    traveler_index INTEGER NOT NULL,
    ptc VARCHAR(10) DEFAULT 'ADT', -- Passenger Type Code (ADT, CHD, INF, etc.)
    first_name VARCHAR(255) NOT NULL,
    last_name VARCHAR(255) NOT NULL,
    date_of_birth DATE,
    gender VARCHAR(20),
    metadata JSONB, -- Additional info like passport, frequent flyer, etc.
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(order_id, traveler_index)
);

CREATE INDEX IF NOT EXISTS idx_travelers_order ON travelers(order_id);
CREATE INDEX IF NOT EXISTS idx_travelers_names ON travelers(last_name, first_name);

-- 2. Enhance ORDERS with Contact Information
ALTER TABLE orders ADD COLUMN IF NOT EXISTS contact_phone VARCHAR(50);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS contact_first_name VARCHAR(255);
ALTER TABLE orders ADD COLUMN IF NOT EXISTS contact_last_name VARCHAR(255);

-- 3. Add DID/Identity support to travelers if not already present (ONE ID)
ALTER TABLE travelers ADD COLUMN IF NOT EXISTS traveler_did VARCHAR(255);
CREATE INDEX IF NOT EXISTS idx_travelers_did ON travelers(traveler_did);
