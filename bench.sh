EXPERIMENT_ID=1 BOT_COUNT=2 INITIAL_SERVER_COUNT=1 docker compose down -v 

experiment=1
for bots_iter in {2..5}; do
  for servers_iter in {1..3}; do
    bots=$((3 * bots_iter))
    servers=$((2 * servers_iter))
    echo "start experiment $experiment with $bots bots and $servers servers..."
    EXPERIMENT_ID=$experiment BOT_COUNT=$bots INITIAL_SERVER_COUNT=$servers timeout 100 docker compose up --build
    experiment=$((experiment + 1))
  done
done
