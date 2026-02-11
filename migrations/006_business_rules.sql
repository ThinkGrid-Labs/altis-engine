-- Configurable Business Rules Schema
-- Allows airlines to customize pricing, bundling, inventory, and products

-- ============================================================================
-- 1. AIRLINES
-- ============================================================================

CREATE TABLE airlines (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(3) UNIQUE NOT NULL,  -- IATA code: 'AA', 'UA', 'BA'
    name VARCHAR(255) NOT NULL,
    country VARCHAR(2),
    status VARCHAR(20) DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_airlines_code ON airlines(code);
CREATE INDEX idx_airlines_status ON airlines(status);

-- ============================================================================
-- 2. BUSINESS RULES (Generic)
-- ============================================================================

CREATE TABLE business_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id) ON DELETE CASCADE,
    rule_type VARCHAR(50) NOT NULL,  -- 'PRICING', 'BUNDLING', 'INVENTORY', 'OFFER'
    rule_name VARCHAR(100) NOT NULL,
    rule_config JSONB NOT NULL,
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    valid_from TIMESTAMPTZ,
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_business_rules_airline ON business_rules(airline_id);
CREATE INDEX idx_business_rules_type ON business_rules(rule_type);
CREATE INDEX idx_business_rules_active ON business_rules(is_active, airline_id);
CREATE INDEX idx_business_rules_validity ON business_rules(valid_from, valid_until);

-- ============================================================================
-- 3. PRODUCTS (Airline-Specific Catalog)
-- ============================================================================

CREATE TABLE products (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id) ON DELETE CASCADE,
    product_type VARCHAR(50) NOT NULL,  -- 'FLIGHT', 'SEAT', 'MEAL', 'BAG', 'LOUNGE', etc.
    product_code VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    base_price_nuc INTEGER NOT NULL,
    currency VARCHAR(3) DEFAULT 'NUC',
    metadata JSONB,  -- Product-specific attributes
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(airline_id, product_code)
);

CREATE INDEX idx_products_airline ON products(airline_id);
CREATE INDEX idx_products_type ON products(product_type);
CREATE INDEX idx_products_active ON products(is_active, airline_id);

-- Example product metadata:
-- SEAT: {"category": "EXTRA_LEGROOM", "legroom_inches": 35, "available_rows": [12,13,14]}
-- MEAL: {"category": "HOT", "dietary": ["VEGETARIAN", "GLUTEN_FREE"]}
-- BAG: {"weight_kg": 23, "dimensions_cm": [56, 36, 23]}

-- ============================================================================
-- 4. PRICING RULES (Specific)
-- ============================================================================

CREATE TABLE pricing_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id) ON DELETE CASCADE,
    product_id UUID REFERENCES products(id) ON DELETE CASCADE,
    rule_name VARCHAR(100) NOT NULL,
    rule_type VARCHAR(50) NOT NULL,  -- 'DEMAND', 'TIME', 'SEASONAL', 'BUNDLE', 'ROUTE'
    conditions JSONB NOT NULL,  -- When to apply this rule
    adjustments JSONB NOT NULL,  -- How to adjust price
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_pricing_rules_airline ON pricing_rules(airline_id);
CREATE INDEX idx_pricing_rules_product ON pricing_rules(product_id);
CREATE INDEX idx_pricing_rules_type ON pricing_rules(rule_type);
CREATE INDEX idx_pricing_rules_active ON pricing_rules(is_active, airline_id);

-- Example conditions:
-- {"product_type": "FLIGHT", "cabin_class": "ECONOMY", "utilization": {"min": 0.7}}
-- {"days_until_departure": {"max": 7}}
-- {"date_range": {"start": "2024-06-01", "end": "2024-08-31"}}

-- Example adjustments:
-- {"type": "MULTIPLIER", "value": 1.5}
-- {"type": "FORMULA", "formula": "1.0 + (utilization^2 * 2.0)", "min": 0.5, "max": 3.0}
-- {"type": "FIXED", "value": 5000}

-- ============================================================================
-- 5. BUNDLE TEMPLATES
-- ============================================================================

CREATE TABLE bundle_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id) ON DELETE CASCADE,
    bundle_name VARCHAR(100) NOT NULL,
    bundle_type VARCHAR(50) NOT NULL,  -- 'ECONOMY', 'COMFORT', 'PREMIUM', 'CUSTOM'
    product_types JSONB NOT NULL,  -- Which product types to include
    discount_percentage DECIMAL(5,2) DEFAULT 0,
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_bundle_templates_airline ON bundle_templates(airline_id);
CREATE INDEX idx_bundle_templates_type ON bundle_templates(bundle_type);
CREATE INDEX idx_bundle_templates_active ON bundle_templates(is_active, airline_id);

-- Example product_types:
-- [
--   {"type": "FLIGHT", "required": true},
--   {"type": "SEAT", "category": "EXTRA_LEGROOM", "required": false},
--   {"type": "MEAL", "category": "HOT", "required": false},
--   {"type": "BAG", "quantity": 1, "required": false}
-- ]

-- ============================================================================
-- 6. INVENTORY RULES
-- ============================================================================

CREATE TABLE inventory_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL,  -- 'SEAT', 'MEAL', 'LOUNGE', 'BAG'
    hold_duration_seconds INTEGER DEFAULT 900,  -- 15 minutes
    overbooking_percentage DECIMAL(5,2) DEFAULT 0,
    min_availability_threshold INTEGER DEFAULT 0,
    auto_release_on_expiry BOOLEAN DEFAULT true,
    notify_on_low_inventory BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_inventory_rules_airline ON inventory_rules(airline_id);
CREATE INDEX idx_inventory_rules_type ON inventory_rules(resource_type);

-- ============================================================================
-- 7. OFFER GENERATION RULES
-- ============================================================================

CREATE TABLE offer_generation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id) ON DELETE CASCADE,
    rule_name VARCHAR(100) NOT NULL,
    max_offers INTEGER DEFAULT 5,
    offer_types JSONB NOT NULL,  -- Which bundle types to generate
    ranking_weights JSONB NOT NULL,  -- How to rank offers
    expiry_minutes INTEGER DEFAULT 15,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_offer_gen_rules_airline ON offer_generation_rules(airline_id);

-- Example offer_types:
-- [
--   {"type": "FLIGHT_ONLY", "priority": 1, "always_include": true},
--   {"type": "COMFORT", "priority": 2, "always_include": true},
--   {"type": "PREMIUM", "priority": 3, "always_include": false}
-- ]

-- Example ranking_weights:
-- {"conversion_probability": 0.6, "profit_margin": 0.4}

-- ============================================================================
-- 8. RULE AUDIT LOG
-- ============================================================================

CREATE TABLE rule_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    airline_id UUID REFERENCES airlines(id),
    rule_id UUID,
    rule_type VARCHAR(50),
    action VARCHAR(20),  -- 'CREATE', 'UPDATE', 'DELETE', 'ACTIVATE', 'DEACTIVATE'
    changed_by VARCHAR(255),
    changes JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_rule_audit_airline ON rule_audit_log(airline_id);
CREATE INDEX idx_rule_audit_created ON rule_audit_log(created_at);

-- ============================================================================
-- SAMPLE DATA
-- ============================================================================

-- Insert sample airline
INSERT INTO airlines (code, name, country) VALUES
('AA', 'American Airlines', 'US'),
('UA', 'United Airlines', 'US'),
('BA', 'British Airways', 'GB');

-- Insert sample products for American Airlines
INSERT INTO products (airline_id, product_type, product_code, name, base_price_nuc, metadata)
SELECT 
    (SELECT id FROM airlines WHERE code = 'AA'),
    'SEAT',
    'SEAT-EXTRA-LEG',
    'Extra Legroom Seat',
    3000,
    '{"category": "EXTRA_LEGROOM", "legroom_inches": 35, "available_rows": [12,13,14,15]}'::jsonb;

INSERT INTO products (airline_id, product_type, product_code, name, base_price_nuc, metadata)
SELECT 
    (SELECT id FROM airlines WHERE code = 'AA'),
    'MEAL',
    'MEAL-HOT',
    'Hot Meal',
    1500,
    '{"category": "HOT", "dietary": ["VEGETARIAN", "CHICKEN", "BEEF"]}'::jsonb;

-- Insert sample pricing rule (Continuous Pricing)
INSERT INTO pricing_rules (airline_id, product_id, rule_name, rule_type, conditions, adjustments, priority)
SELECT 
    (SELECT id FROM airlines WHERE code = 'AA'),
    (SELECT id FROM products WHERE product_code = 'SEAT-EXTRA-LEG' LIMIT 1),
    'Continuous Pricing - Seats',
    'DEMAND',
    '{"product_type": "SEAT", "utilization": {"min": 0.0}}'::jsonb,
    '{"type": "FORMULA", "formula": "1.0 + (utilization^2 * 2.0)", "min": 0.5, "max": 3.0}'::jsonb,
    10;

-- Insert sample bundle template (Comfort Bundle)
INSERT INTO bundle_templates (airline_id, bundle_name, bundle_type, product_types, discount_percentage, priority)
SELECT 
    (SELECT id FROM airlines WHERE code = 'AA'),
    'Comfort Bundle',
    'COMFORT',
    '[
        {"type": "FLIGHT", "required": true},
        {"type": "SEAT", "category": "EXTRA_LEGROOM", "required": false},
        {"type": "MEAL", "category": "HOT", "required": false},
        {"type": "BAG", "quantity": 1, "required": false}
    ]'::jsonb,
    10.0,
    2;

-- Insert sample inventory rule
INSERT INTO inventory_rules (airline_id, resource_type, hold_duration_seconds, overbooking_percentage)
SELECT 
    (SELECT id FROM airlines WHERE code = 'AA'),
    'SEAT',
    900,
    5.0;
