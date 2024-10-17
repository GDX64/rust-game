imageName="br_server"
userName="gdx64"
tagName=$userName/$imageName

source ~/.bashrc

function buildDockerImage(){
    echo "Building docker image"
    sudo docker build --platform linux/amd64 -t $imageName .
    sudo docker save -o img.tar $imageName
}

function runImage(){
    echo "Running docker image"
    ## stop all docker containers running
    sudo docker stop $(sudo docker ps -a -q)
    sudo docker pull $tagName
    sudo docker volume create archpelagus
    sudo docker run --name archpelagus --rm -d -p 5000:5000 -v archpelagus:/data $tagName 
}

function uploadImage(){
    echo "Uploading docker image"
    sudo docker tag $imageName $tagName
    sudo docker push $tagName
}

if [ "$1" == "run" ]; then
    runImage
    exit 0
elif [ "$1" == "build" ]; then
    buildDockerImage
    uploadImage
    exit 0
fi
