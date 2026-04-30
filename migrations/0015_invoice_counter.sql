CREATE TABLE invoice_counters (
    month TEXT PRIMARY KEY,
    last_seq INTEGER NOT NULL DEFAULT 0
);

-- Initialize from existing invoice data so we don't reuse numbers
INSERT INTO invoice_counters (month, last_seq)
SELECT
    SUBSTRING(invoice_no, 5, 7) AS month,
    MAX(CAST(SUBSTRING(invoice_no, 13) AS INTEGER)) AS last_seq
FROM invoices
WHERE invoice_no ~ '^INV-\d{4}-\d{2}-\d{4}$'
GROUP BY SUBSTRING(invoice_no, 5, 7)
ON CONFLICT (month) DO NOTHING;
