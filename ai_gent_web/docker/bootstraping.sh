#sqlx database create
#sqlx migrate run

# psql -c 'drop database ai_gent;'
/usr/lib/postgresql/14/bin/psql -c 'create database ai_gent;'
/usr/lib/postgresql/14/bin/psql -d ai_gent -f bootstraping.sql

