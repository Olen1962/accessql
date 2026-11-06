DROP TABLE IF EXISTS encomendas;
DROP TABLE IF EXISTS clientes;

CREATE TABLE clientes (
    id INTEGER PRIMARY KEY,
    nome TEXT NOT NULL,
    email TEXT,
    ativo INTEGER NOT NULL DEFAULT 1
) STRICT;

CREATE TABLE encomendas (
    id INTEGER PRIMARY KEY,
    cliente_id INTEGER REFERENCES clientes(id),
    total REAL
) STRICT;

INSERT INTO clientes (nome, email) VALUES ('Ana', 'ana@exemplo.pt'), ('Rui', NULL);
INSERT INTO encomendas (cliente_id, total) VALUES (1, 25.5), (1, 10.0), (2, 99.9);
