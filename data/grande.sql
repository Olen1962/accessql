DROP TABLE IF EXISTS numeros;
DROP TABLE IF EXISTS vazia;

CREATE TABLE numeros (
    n INTEGER PRIMARY KEY,
    quadrado INTEGER,
    texto TEXT,
    valor REAL,
    bin BLOB
) STRICT;

CREATE TABLE vazia (id INTEGER PRIMARY KEY, nome TEXT) STRICT;

WITH RECURSIVE s(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM s WHERE n < 2000)
INSERT INTO numeros
SELECT n, n * n, 'linha ' || n, n / 3.0, x'DEADBEEF' FROM s;
