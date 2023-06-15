CREATE USER docker WITH PASSWORD 'password' CREATEDB;

CREATE DATABASE lacpass_trusted_list_development
WITH OWNER = docker
CONNECTION LIMIT = -1;

CREATE DATABASE lacpass_trusted_list
WITH OWNER = docker
CONNECTION LIMIT = -1;

