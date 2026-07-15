-- Sample TPC-H-like schema
CREATE TABLE region (
                        r_regionkey INT PRIMARY KEY,
                        r_name CHAR(25),
                        r_comment VARCHAR(152)
);

CREATE TABLE nation (
                        n_nationkey INT PRIMARY KEY,
                        n_name CHAR(25),
                        n_regionkey INT REFERENCES region(r_regionkey),
                        n_comment VARCHAR(152)
);

CREATE TABLE supplier (
                          s_suppkey INT PRIMARY KEY,
                          s_name CHAR(25),
                          s_address VARCHAR(40),
                          s_nationkey INT REFERENCES nation(n_nationkey),
                          s_phone CHAR(15),
                          s_acctbal DECIMAL(15,2),
                          s_comment VARCHAR(101)
);

CREATE TABLE part (
                      p_partkey INT PRIMARY KEY,
                      p_name VARCHAR(55),
                      p_mfgr CHAR(25),
                      p_brand CHAR(10),
                      p_type VARCHAR(25),
                      p_size INT,
                      p_container CHAR(10),
                      p_retailprice DECIMAL(15,2),
                      p_comment VARCHAR(23)
);

CREATE TABLE partsupp (
                          ps_partkey INT REFERENCES part(p_partkey),
                          ps_suppkey INT REFERENCES supplier(s_suppkey),
                          ps_availqty INT,
                          ps_supplycost DECIMAL(15,2),
                          ps_comment VARCHAR(199),
                          PRIMARY KEY (ps_partkey, ps_suppkey)
);

CREATE TABLE customer (
                          c_custkey INT PRIMARY KEY,
                          c_name VARCHAR(25),
                          c_address VARCHAR(40),
                          c_nationkey INT REFERENCES nation(n_nationkey),
                          c_phone CHAR(15),
                          c_acctbal DECIMAL(15,2),
                          c_mktsegment CHAR(10),
                          c_comment VARCHAR(117)
);

CREATE TABLE orders (
                        o_orderkey INT PRIMARY KEY,
                        o_custkey INT REFERENCES customer(c_custkey),
                        o_orderstatus CHAR(1),
                        o_totalprice DECIMAL(15,2),
                        o_orderdate DATE,
                        o_orderpriority CHAR(15),
                        o_clerk CHAR(15),
                        o_shippriority INT,
                        o_comment VARCHAR(79)
);

CREATE TABLE lineitem (
                          l_orderkey INT REFERENCES orders(o_orderkey),
                          l_partkey INT REFERENCES part(p_partkey),
                          l_suppkey INT REFERENCES supplier(s_suppkey),
                          l_linenumber INT,
                          l_quantity DECIMAL(15,2),
                          l_extendedprice DECIMAL(15,2),
                          l_discount DECIMAL(15,2),
                          l_tax DECIMAL(15,2),
                          l_returnflag CHAR(1),
                          l_linestatus CHAR(1),
                          l_shipdate DATE,
                          l_commitdate DATE,
                          l_receiptdate DATE,
                          l_shipinstruct CHAR(25),
                          l_shipmode CHAR(10),
                          l_comment VARCHAR(44),
                          PRIMARY KEY (l_orderkey, l_linenumber)
);

-- Insert small sample data (scale factor ~0.01)
INSERT INTO region VALUES (1, 'AMERICA', 'Test'), (2, 'ASIA', 'Test');
INSERT INTO nation VALUES (1, 'USA', 1, 'Test'), (2, 'CHINA', 2, 'Test');
INSERT INTO supplier SELECT s, 'Supplier'||s, 'Address'||s, 1+(s%2), 'phone', 100.0, 'comment' FROM generate_series(1,100) s;
INSERT INTO part SELECT p, 'Part'||p, 'MFGR', 'Brand', 'Type', 10, 'Box', 100.0, 'comment' FROM generate_series(1,500) p;
INSERT INTO partsupp SELECT ps, 1+(ps%100), ps, 10.0, 'comment' FROM generate_series(1,200) ps;
INSERT INTO customer SELECT c, 'Customer'||c, 'Address', 1+(c%2), 'phone', 0.0, 'BUILDING', 'comment' FROM generate_series(1,300) c;
INSERT INTO orders SELECT o, 1+(o%300), 'O', 1000.0, '2024-01-01'::DATE + (o%365), 'HIGH', 'Clerk', 0, 'comment' FROM generate_series(1,1000) o;
INSERT INTO lineitem SELECT o, 1+(o%500), 1+(o%100), o%7, 10, 100.0, 0.05, 0.02, 'R', 'F', '2024-01-01'::DATE, '2024-01-01'::DATE, '2024-01-01'::DATE, 'DELIVER', 'TRUCK', 'comment' FROM generate_series(1,5000) o;