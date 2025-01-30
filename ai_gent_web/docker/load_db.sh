su - postgres -c "nohup /usr/lib/postgresql/14/bin/postgres --config-file=/etc/postgresql/14/main/postgresql.conf &"
sleep 2
su - postgres -c "/usr/lib/postgresql/14/bin/psql -c \"CREATE USER db_user WITH PASSWORD 'dslakdasd' CREATEDB\""
su - postgres -c "/usr/lib/postgresql/14/bin/psql -c \"CREATE DATABASE db_user OWNER db_user;\""
sleep 2
export PGUSER=db_user
export PGPASSWORD=dslakdasd
su - db_user -c "cd /opt/database/ && bash bootstraping.sh"
