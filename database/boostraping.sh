# psql -c 'create database ai_gent;'
sqlx database create
sqlx migrate run

psql -d ai_gent -a -f boostraping.sql

