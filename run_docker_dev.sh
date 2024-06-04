#store all running containers in a variable

allRunning=$(docker ps -a -q)

docker stop $allRunning
docker rm $allRunning

SOCKET_SERVER="ws://localhost:5000/ws"

docker build -t br_server --build-arg FRONT_SERVER=$SOCKET_SERVER .
docker run --rm -d -p 5000:5000 -p 3000:3000 br_server
