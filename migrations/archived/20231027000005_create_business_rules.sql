CREATE TABLE business_rules (
    rule_key VARCHAR(255) PRIMARY KEY,
    rule_value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed initial defaults (Optional, but good for CMS visibility context)
INSERT INTO business_rules (rule_key, rule_value, description) VALUES
('pricing_multiplier', '{"value": 1.0, "type": "float"}', 'Global pricing multiplier (1.0 = 100%)'),
('pricing_adjustment', '{"value": 0.0, "type": "float"}', 'Global fixed price adjustment'),
('tax_rate', '{"value": 0.10, "type": "float"}', 'Global tax rate (0.10 = 10%)'),
('booking_fee', '{"value": 5.0, "type": "float"}', 'Fixed booking fee per transaction');
