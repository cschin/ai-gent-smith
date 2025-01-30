cp -a ../../database/migrations .
cp -a ../../database/boostraping.sh .
cp ../target/release/ai_agent_smith .
docker build -t test .
