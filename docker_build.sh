imageName="br_server"

function buildDockerImage(){
    docker build -t $imageName .
}

buildDockerImage