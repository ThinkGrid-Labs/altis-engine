-- Add interline support to order_items
ALTER TABLE order_items 
ADD COLUMN operating_carrier_id UUID,
ADD COLUMN net_rate_nuc INTEGER,
ADD COLUMN commission_nuc INTEGER;

-- Comment on columns for clarity
COMMENT ON COLUMN order_items.operating_carrier_id IS 'The carrier operating the service (Supplier)';
COMMENT ON COLUMN order_items.net_rate_nuc IS 'The amount owed to the operating carrier after commissions';
COMMENT ON COLUMN order_items.commission_nuc IS 'The commission kept by the retailer';
