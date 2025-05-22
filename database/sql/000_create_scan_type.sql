CREATE TABLE scan_type (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
name varchar(255) NOT NULL,
description varchar(1000)
);
