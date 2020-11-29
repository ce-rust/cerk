# Common Docker Image for CERK 

## Build & Run

Without any config changes hello world example will be executed.

1. `docker build . -t cerk-common`
2. `docker run cerk-common`

## Configure

Mount a custom `config.json` and `init.json` into the docker container.
