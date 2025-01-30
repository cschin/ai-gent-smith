#sqlx database create
#sqlx migrate run

# psql -c 'drop database ai_gent;'
psql -c 'create database ai_gent;'
psql -d ai_gent -f bootstraping.sql

