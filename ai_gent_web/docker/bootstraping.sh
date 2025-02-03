#sqlx database create
#sqlx migrate run

# psql -c 'drop database ai_gent;'
/usr/lib/postgresql/16/bin/psql -c 'create database ai_gent;'
/usr/lib/postgresql/16/bin/psql -d ai_gent -f bootstraping.sql

