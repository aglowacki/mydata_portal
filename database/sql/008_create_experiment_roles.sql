CREATE TABLE experiment_roles (
id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
type varchar(255) NOT NULL,
description varchar(1000)
);
