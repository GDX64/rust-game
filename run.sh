#store all running containers in a variable

allRunning=$(docker ps -a -q)

docker stop $allRunning
docker rm $allRunning

cd backend
docker build -t backend .
docker run -d -p 5000:5000 backend

cd ../front

docker build -t front .
docker run -d -p 3000:3000 front

