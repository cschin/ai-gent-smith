su - postgres -c "nohup /usr/lib/postgresql/14/bin/postgres --config-file=/etc/postgresql/14/main/postgresql.conf &" 
sleep 2
su - postgres -c "psql -c \"CREATE USER db_user WITH PASSWORD 'dslakdasd' CREATEDB;\""
sleep 2
cd /opt/ai-gent-smith/database && . $HOME/.cargo/env && sqlx database create && sqlx migrate run
