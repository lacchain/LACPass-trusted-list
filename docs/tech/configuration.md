# LACPass Trusted List

LacPass Trusted List is the component that allows getting all public keys that are trusted for the sake of verifying signed information. Public keys are gathered from the Lacchain Trusted Registry as well as from other sources authorized by the Governance.

## Basic Requirements

1. `cp .example.env.dev .env.dev`
1. `cp .example.env .env.prod`
1. `cp .example.env.test .env.test`

## Some scripts

## Github Actions

## Running with Docker

### Prerequisites

In order to run the app with Docker, you should install or update to the latest version, we recommend to install [Docker-Desktop](https://docs.docker.com/get-docker/) due to composer and some cool CLI-UI tools are included.

## Running directly

- Prepare databases

```sh
docker-compose -f docker-compose-dbs.yml --env-file .env.dev up
```

- Prepare environment variables and run

```sh
cp .example.env.dev.sh .env.dev.sh
chmod +x .env.dev.sh
. ./.env.dev.sh
RUST_LOG="info" cargo run # RUST_LOG="info" cargo watch -x run (if you have "watch" installed)
```

### Development with Docker

The following commands will build and run all you need to start working on the base, without any other installation requirements. Important: if you already have postgres running locally, you'll need to kill the service before run `docker-compose up`.

Note: Don't forget to create a copy of the env file as mentioned in the **basic requirements** section.

```
docker network create backend
```

```
docker-compose -f docker-compose-dev.yml --env-file .env.dev build
```

```
docker-compose -f docker-compose-dev.yml --env-file .env.dev up
```

### Testing Docker production images with docker-compose

Note: Don't forget to create a copy of the env file as mentioned in the **basic requirements** section.

The following commands will build and run all you need to start working on the base, without any other installation requirements. Important: if you already have postgres running locally, you'll need to kill the service before run `docker-compose up`.

```
docker network create backend
```

```
docker-compose -f docker-compose.yml --env-file .env.prod build
```

```
docker-compose -f docker-compose.yml --env-file .env.prod up
```

You can also run it with a configured image:

```
docker-compose -f docker-compose-image.yml --env-file .env.prod up
```

### Deployment with Docker (only for production)

The following commands will `build and run` a Docker image ready for production and size-optimized.

#### Build Docker image

```
docker build -f Dockerfile.prod -t lacpass-trusted-list .
```

#### Run docker image (you need to add .env file as param)

```
docker run --rm --env-file=.env.prod -p 3000:3000 --name rust-api lacpass-trusted-list
```

## Debugging

## API Documentation

After running the server you will find OpenAPI Specification here: `http://<host>:<port>/docs`

## App scaffolding

## Dependencies

## Code Quality

TODO: SonarQube

## Migrations and New Entities generation

[Migration Stuffs](./migrations.md)
