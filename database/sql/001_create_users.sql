CREATE TABLE users (
badge INT NOT NULL PRIMARY KEY,
username VARCHAR (50) NOT NULL,
first_name VARCHAR (50) NOT NULL,
last_name VARCHAR (50) NOT NULL,
institution VARCHAR (400) NOT NULL,
email VARCHAR (50) NOT NULL,
user_access_control_id integer NOT NULL REFERENCES user_access_control (id)
);
