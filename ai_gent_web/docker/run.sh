su - postgres -c "nohup /usr/lib/postgresql/14/bin/postgres --config-file=/etc/postgresql/14/main/postgresql.conf&"
sleep 2
su - db_user -c "OPENAI_API_KEY=$OPENAI_API_KEY db_PGUSER=db_user PGPASSWORD=dslakdasd DATABASE_URL=postgres://db_user@localhost/ai_gent /opt/bin/ai_gent_web"
