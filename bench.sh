EXPERIMENT_ID=1 BOT_COUNT=2 INITIAL_SERVER_COUNT=1 docker compose down  
docker compose -f docker-compose-analyze.yml down 

experiment=1
for servers in 1 2 4 6 8 10; do
  for bots in 2 4 6 8 10; do
      echo "start experiment $experiment with $bots bots and $servers servers..."
      experiment=$((experiment + 1))
      EXPERIMENT_ID=$experiment BOT_COUNT=$bots INITIAL_SERVER_COUNT=$servers timeout 30 docker compose up --build 
  done
done

docker compose -f docker-compose-analyze.yml up --build