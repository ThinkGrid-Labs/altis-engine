-- Offers and Orders Schema
-- Core tables for the Offer/Order dynamic retailing system

-- ============================================================================
-- 1. OFFERS
-- ============================================================================

CREATE TABLE offers (
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

CREATE INDEX idx_offers_customer ON offers(customer_id);
CREATE INDEX idx_offers_airline ON offers(airline_id);
CREATE INDEX idx_offers_status ON offers(status);
CREATE INDEX idx_offers_expires ON offers(expires_at) WHERE status = 'ACTIVE';
CREATE INDEX idx_offers_created ON offers(created_at);

-- ============================================================================
-- 2. OFFER ITEMS
-- ============================================================================

CREATE TABLE offer_items (
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

CREATE INDEX idx_offer_items_offer ON offer_items(offer_id);
CREATE INDEX idx_offer_items_product ON offer_items(product_id);
CREATE INDEX idx_offer_items_type ON offer_items(product_type);

-- ============================================================================
-- 3. ORDERS
-- ============================================================================

CREATE TABLE orders (
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

CREATE INDEX idx_orders_customer ON orders(customer_id);
CREATE INDEX idx_orders_email ON orders(customer_email);
CREATE INDEX idx_orders_offer ON orders(offer_id);
CREATE INDEX idx_orders_airline ON orders(airline_id);
CREATE INDEX idx_orders_status ON orders(status);
CREATE INDEX idx_orders_created ON orders(created_at);

-- ============================================================================
-- 4. ORDER ITEMS
-- ============================================================================

CREATE TABLE order_items (
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

CREATE INDEX idx_order_items_order ON order_items(order_id);
CREATE INDEX idx_order_items_product ON order_items(product_id);
CREATE INDEX idx_order_items_type ON order_items(product_type);
CREATE INDEX idx_order_items_status ON order_items(status);

-- ============================================================================
-- 5. FULFILLMENT
-- ============================================================================

CREATE TABLE fulfillment (
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

CREATE INDEX idx_fulfillment_order ON fulfillment(order_id);
CREATE INDEX idx_fulfillment_item ON fulfillment(order_item_id);
CREATE INDEX idx_fulfillment_barcode ON fulfillment(barcode);

-- ============================================================================
-- 6. ORDER CHANGES (Audit Trail)
-- ============================================================================

CREATE TABLE order_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID REFERENCES orders(id) ON DELETE CASCADE,
    change_type VARCHAR(50) NOT NULL,  -- STATUS_CHANGE, ITEM_ADDED, ITEM_REFUNDED, PAYMENT
    old_value JSONB,
    new_value JSONB,
    changed_by VARCHAR(255),
    reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_order_changes_order ON order_changes(order_id);
CREATE INDEX idx_order_changes_type ON order_changes(change_type);
CREATE INDEX idx_order_changes_created ON order_changes(created_at);

-- ============================================================================
-- 7. SEAT ASSIGNMENTS (Specific to Flight Products)
-- ============================================================================

CREATE TABLE seat_assignments (
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

CREATE INDEX idx_seat_assignments_order ON seat_assignments(order_id);
CREATE INDEX idx_seat_assignments_flight ON seat_assignments(flight_id);
CREATE INDEX idx_seat_assignments_seat ON seat_assignments(flight_id, seat_number);

-- ============================================================================
-- SAMPLE DATA
-- ============================================================================

-- Insert sample offer (assuming airline AA exists from 006_business_rules.sql)
INSERT INTO offers (id, customer_id, airline_id, search_context, total_nuc, expires_at)
SELECT 
    gen_random_uuid(),
    'customer-123',
    (SELECT id FROM airlines WHERE code = 'AA' LIMIT 1),
    '{"origin": "JFK", "destination": "LHR", "date": "2024-06-01", "passengers": 2}'::jsonb,
    74000,
    NOW() + INTERVAL '15 minutes'
WHERE EXISTS (SELECT 1 FROM airlines WHERE code = 'AA');

-- Insert sample offer items
INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT 
    (SELECT id FROM offers WHERE customer_id = 'customer-123' LIMIT 1),
    'FLIGHT',
    'UA300',
    'United Airlines UA300 JFK-LHR',
    36000,
    '{"flight_number": "UA300", "departure": "2024-06-01T12:00:00Z", "arrival": "2024-06-02T00:00:00Z"}'::jsonb
WHERE EXISTS (SELECT 1 FROM offers WHERE customer_id = 'customer-123');

INSERT INTO offer_items (offer_id, product_type, product_code, name, price_nuc, metadata)
SELECT 
    (SELECT id FROM offers WHERE customer_id = 'customer-123' LIMIT 1),
    'FLIGHT',
    'UA301',
    'United Airlines UA301 LHR-JFK',
    38000,
    '{"flight_number": "UA301", "departure": "2024-06-08T13:00:00Z", "arrival": "2024-06-08T16:00:00Z"}'::jsonb
WHERE EXISTS (SELECT 1 FROM offers WHERE customer_id = 'customer-123');
