su - postgres -c "nohup /usr/lib/postgresql/16/bin/postgres --config-file=/etc/postgresql/16/main/postgresql.conf &" 
sleep 2
su - postgres -c "psql -c \"CREATE USER db_user WITH PASSWORD 'dslakdasd' CREATEDB;\""
sleep 2
su - postgres -c "/usr/lib/postgresql/16/bin/psql -c \"ALTER USER db_user WITH SUPERUSER;\""
sleep 2
su - postgres -c "/usr/lib/postgresql/16/bin/psql -c \"GRANT USAGE ON SCHEMA pg_catalog TO db_user;\""
sleep 2
cd /opt/ai-gent-smith/database && . $HOME/.cargo/env && sqlx database create && sqlx migrate run
