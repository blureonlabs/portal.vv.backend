CREATE TYPE document_type AS ENUM ('license', 'visa', 'passport', 'emirates_id', 'medical', 'registration_card', 'insurance_certificate', 'receipt', 'other');
CREATE TYPE document_entity AS ENUM ('driver', 'vehicle');

CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type document_entity NOT NULL,
    entity_id UUID NOT NULL,
    doc_type document_type NOT NULL,
    file_url TEXT NOT NULL,
    file_name TEXT NOT NULL,
    expiry_date DATE,
    uploaded_by UUID NOT NULL REFERENCES profiles(id),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_documents_entity ON documents(entity_type, entity_id);
CREATE INDEX idx_documents_expiry ON documents(expiry_date) WHERE expiry_date IS NOT NULL;
