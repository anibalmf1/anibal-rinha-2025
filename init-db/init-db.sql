CREATE UNLOGGED TABLE payments (
    correlation_id UUID PRIMARY KEY,
    payment_processor VARCHAR(50) NOT NULL,
    amount DECIMAL NOT NULL,
    requested_at TIMESTAMP NOT NULL
);

CREATE INDEX payments_requested_at ON payments (requested_at);
CREATE INDEX payments_payment_processor ON payments (payment_processor);
