EXPERIMENT_ID=1 BOT_COUNT=2 INITIAL_SERVER_COUNT=1 docker compose down -v 

EXPERIMENT_ID=1 BOT_COUNT=10 INITIAL_SERVER_COUNT=2 docker compose up --build --abort-on-container-exit
# experiment=1
# for bots_iter in {2..6}; do
#   for servers_iter in {1..4}; do
#     bots=$((3 * bots_iter))
#     servers=$((2 * servers_iter))
#     echo "start experiment $experiment with $bots bots and $servers servers..."
#     EXPERIMENT_ID=$experiment BOT_COUNT=$bots INITIAL_SERVER_COUNT=$servers docker compose up --build --abort-on-container-exit
#     experiment=$((experiment + 1))
#   done
# done
