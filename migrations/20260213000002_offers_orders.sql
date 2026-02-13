-- Offers and Orders Schema
-- Core tables for the Offer/Order dynamic retailing system

-- ============================================================================
-- 1. OFFERS
-- ============================================================================

CREATE TABLE IF NOT EXISTS offers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id VARCHAR(255),  -- Optional: can be anonymous
    airline_id UUID REFERENCES airlines(id),
    search_context JSONB NOT NULL,  -- Origin, destination, dates, passengers
    total_nuc INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'NUC',
    status VARCHAR(20) DEFAULT 'ACTIVE',  -- ACTIVE, EXPIRED, ACCEPTED
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_offers_customer ON offers(customer_id);
CREATE INDEX IF NOT EXISTS idx_offers_airline ON offers(airline_id);
CREATE INDEX IF NOT EXISTS idx_offers_status ON offers(status);
CREATE INDEX IF NOT EXISTS idx_offers_expires ON offers(expires_at) WHERE status = 'ACTIVE';
CREATE INDEX IF NOT EXISTS idx_offers_created ON offers(created_at);

-- ============================================================================
-- 2. OFFER ITEMS
-- ============================================================================

CREATE TABLE IF NOT EXISTS offer_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    offer_id UUID REFERENCES offers(id) ON DELETE CASCADE,
    product_id UUID REFERENCES products(id),
    product_type VARCHAR(50) NOT NULL,  -- FLIGHT, SEAT, MEAL, BAG, etc.
    product_code VARCHAR(50),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price_nuc INTEGER NOT NULL,
    quantity INTEGER DEFAULT 1,
    metadata JSONB,  -- Product-specific data (flight details, seat number, etc.)
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_offer_items_offer ON offer_items(offer_id);
CREATE INDEX IF NOT EXISTS idx_offer_items_product ON offer_items(product_id);
CREATE INDEX IF NOT EXISTS idx_offer_items_type ON offer_items(product_type);

-- ============================================================================
-- 3. ORDERS
-- ============================================================================

CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255),
    offer_id UUID REFERENCES offers(id),
    airline_id UUID REFERENCES airlines(id),
    status VARCHAR(20) NOT NULL,  -- PROPOSED, PAID, FULFILLED, CANCELLED, REFUNDED
    total_nuc INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'NUC',
    payment_method VARCHAR(50),
    payment_reference VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_orders_customer ON orders(customer_id);
CREATE INDEX IF NOT EXISTS idx_orders_email ON orders(customer_email);
CREATE INDEX IF NOT EXISTS idx_orders_offer ON orders(offer_id);
CREATE INDEX IF NOT EXISTS idx_orders_airline ON orders(airline_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_orders_created ON orders(created_at);

-- ============================================================================
-- 4. ORDER ITEMS
-- ============================================================================

CREATE TABLE IF NOT EXISTS order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID REFERENCES products(id),
    product_type VARCHAR(50) NOT NULL,
    product_code VARCHAR(50),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price_nuc INTEGER NOT NULL,
    quantity INTEGER DEFAULT 1,
    status VARCHAR(20) DEFAULT 'ACTIVE',  -- ACTIVE, REFUNDED, CANCELLED
    metadata JSONB,  -- Customization data (seat assignments, meal preferences, etc.)
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_order_items_order ON order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product ON order_items(product_id);
CREATE INDEX IF NOT EXISTS idx_order_items_type ON order_items(product_type);
CREATE INDEX IF NOT EXISTS idx_order_items_status ON order_items(status);

-- ============================================================================
-- 5. FULFILLMENT
-- ============================================================================

CREATE TABLE IF NOT EXISTS fulfillment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    order_item_id UUID REFERENCES order_items(id) ON DELETE CASCADE,
    fulfillment_type VARCHAR(50) NOT NULL,  -- BARCODE, QR_CODE, EMAIL, SMS
    barcode VARCHAR(255) UNIQUE,
    qr_code_data TEXT,
    delivery_method VARCHAR(50),  -- EMAIL, SMS, APP
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_fulfillment_order ON fulfillment(order_id);
CREATE INDEX IF NOT EXISTS idx_fulfillment_item ON fulfillment(order_item_id);
CREATE INDEX IF NOT EXISTS idx_fulfillment_barcode ON fulfillment(barcode);

-- ============================================================================
-- 6. ORDER CHANGES (Audit Trail)
-- ============================================================================

CREATE TABLE IF NOT EXISTS order_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    change_type VARCHAR(50) NOT NULL,  -- STATUS_CHANGE, ITEM_ADDED, ITEM_REFUNDED, PAYMENT
    old_value JSONB,
    new_value JSONB,
    changed_by VARCHAR(255),
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_order_changes_order ON order_changes(order_id);
CREATE INDEX IF NOT EXISTS idx_order_changes_type ON order_changes(change_type);
CREATE INDEX IF NOT EXISTS idx_order_changes_created ON order_changes(created_at);

-- ============================================================================
-- 7. SEAT ASSIGNMENTS (Specific to Flight Products)
-- ============================================================================

CREATE TABLE IF NOT EXISTS seat_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    order_item_id UUID REFERENCES order_items(id) ON DELETE CASCADE,
    flight_id VARCHAR(255) NOT NULL,
    seat_number VARCHAR(10) NOT NULL,
    passenger_index INTEGER NOT NULL,
    passenger_name VARCHAR(255),
    status VARCHAR(20) DEFAULT 'ASSIGNED',  -- ASSIGNED, RELEASED
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(flight_id, seat_number, order_id)
);

CREATE INDEX IF NOT EXISTS idx_seat_assignments_order ON seat_assignments(order_id);
CREATE INDEX IF NOT EXISTS idx_seat_assignments_flight ON seat_assignments(flight_id);
CREATE INDEX IF NOT EXISTS idx_seat_assignments_seat ON seat_assignments(flight_id, seat_number);

-- ============================================================================
-- ============================================================================
-- SAMPLE DATA: LCC FOCUS (HUB: SINGAPORE)
-- ============================================================================

-- Insert sample offers for AirAltis LCC (Origin: SIN)
INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, expires_at)
SELECT gen_random_uuid(), 'traveler-001', id, '{"origin": "SIN", "destination": "MNL", "date": "2024-07-01", "passengers": 1}'::jsonb, 12000, NOW() + INTERVAL '15 minutes'
FROM airlines WHERE code = 'AL'
ON CONFLICT DO NOTHING;

INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, expires_at)
SELECT gen_random_uuid(), 'traveler-002', id, '{"origin": "SIN", "destination": "BKK", "date": "2024-07-05", "passengers": 1}'::jsonb, 9500, NOW() + INTERVAL '15 minutes'
FROM airlines WHERE code = 'AL'
ON CONFLICT DO NOTHING;

INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, expires_at)
SELECT gen_random_uuid(), 'traveler-003', id, '{"origin": "SIN", "destination": "KUL", "date": "2024-07-10", "passengers": 1}'::jsonb, 4500, NOW() + INTERVAL '15 minutes'
FROM airlines WHERE code = 'AL'
ON CONFLICT DO NOTHING;

INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, expires_at)
SELECT gen_random_uuid(), 'traveler-004', id, '{"origin": "SIN", "destination": "CGK", "date": "2024-07-15", "passengers": 1}'::jsonb, 8500, NOW() + INTERVAL '15 minutes'
FROM airlines WHERE code = 'AL'
ON CONFLICT DO NOTHING;

INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, expires_at)
SELECT gen_random_uuid(), 'traveler-005', id, '{"origin": "SIN", "destination": "SGN", "date": "2024-07-20", "passengers": 1}'::jsonb, 7500, NOW() + INTERVAL '15 minutes'
FROM airlines WHERE code = 'AL'
ON CONFLICT DO NOTHING;

-- Insert sample offer items (Unbundled LCC Model)

-- traveler-001: SIN-MNL + Ancillaries
INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT id, 'FLIGHT', 'AL101', 'AirAltis AL101 SIN-MNL', 8500, '{"flight_number": "AL101"}'::jsonb
FROM offers WHERE customer_id = 'traveler-001'
ON CONFLICT DO NOTHING;

INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT id, 'SEAT', 'LCC-SEAT-FRONTRW', 'Front Row Seat upgrade', 2500, '{"seat": "1A"}'::jsonb
FROM offers WHERE customer_id = 'traveler-001'
ON CONFLICT DO NOTHING;

INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT id, 'MEAL', 'LCC-MEAL-SNACK', 'Ham & Cheese Sandwich', 1000, '{"description": "Pre-purchased onboard snack"}'::jsonb
FROM offers WHERE customer_id = 'traveler-001'
ON CONFLICT DO NOTHING;

-- traveler-002: SIN-BKK
INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT id, 'FLIGHT', 'AL202', 'AirAltis AL202 SIN-BKK', 7000, '{"flight_number": "AL202"}'::jsonb
FROM offers WHERE customer_id = 'traveler-002';

-- traveler-003: SIN-KUL
INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT id, 'FLIGHT', 'AL303', 'AirAltis AL303 SIN-KUL', 4000, '{"flight_number": "AL303"}'::jsonb
FROM offers WHERE customer_id = 'traveler-003';

-- traveler-004: SIN-CGK
INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT id, 'FLIGHT', 'AL404', 'AirAltis AL404 SIN-CGK', 7500, '{"flight_number": "AL404"}'::jsonb
FROM offers WHERE customer_id = 'traveler-004';

-- traveler-005: SIN-SGN
INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT id, 'FLIGHT', 'AL505', 'AirAltis AL505 SIN-SGN', 6500, '{"flight_number": "AL505"}'::jsonb
FROM offers WHERE customer_id = 'traveler-005';
