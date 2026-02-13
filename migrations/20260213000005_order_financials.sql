-- Add revenue_status to order_items
ALTER TABLE order_items ADD COLUMN revenue_status VARCHAR(20) DEFAULT 'unearned';

-- Create order_ledger table for financial audit
CREATE TABLE order_ledger (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id),
    order_item_id UUID NOT NULL REFERENCES order_items(id),
    transaction_type VARCHAR(50) NOT NULL, -- REVENUE_RECOGNITION, REFUND, ADJUSTMENT
    amount_nuc INTEGER NOT NULL,
    currency VARCHAR(10) DEFAULT 'NUC',
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Add index for financial reporting
CREATE INDEX idx_ledger_order_id ON order_ledger(order_id);
CREATE INDEX idx_order_items_revenue ON order_items(revenue_status);
