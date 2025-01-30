su - postgres -c "nohup /usr/lib/postgresql/16/bin/postgres --config-file=/etc/postgresql/16/main/postgresql.conf&"
sleep 2
/opt/ai-gent-smith/target/release/ai_gent_web
