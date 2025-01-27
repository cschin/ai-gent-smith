psql -c 'create database ai_gent;'
sqlx migrate run

psql -d ai_gent -a -f boostraping.sql

