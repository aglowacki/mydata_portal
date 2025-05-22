CREATE TABLE users (
badge INT UNIQUE NOT NULL,
username VARCHAR (50) NOT NULL,
first_name VARCHAR (50) NOT NULL,
last_name VARCHAR (50) NOT NULL,
institution VARCHAR (400) NOT NULL,
email VARCHAR (50) NOT NULL,
user_type_id integer REFERENCES user_types (id)
);
