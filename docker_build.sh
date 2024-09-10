imageName="br_server"

function buildDockerImage(){
    docker build --platform linux/amd64 -t $imageName .
    docker save -o img.tar $imageName
}

function runImage(){
    ## stop all docker containers running
    sudo docker stop $(docker ps -a -q)
    sudo docker load -i ~/deploys/img.tar
    sudo docker run -d -p 5000:5000 $imageName
}

buildDockerImage
copyImageToServer
runImageOnServer
