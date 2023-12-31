#PORT

export PORT=3025 # use port 80 when running with docker
export EXPOSED_CONTAINER_SERVER_PORT=3025

#TYPEORM

export TYPEORM_TYPE=postgres
export TYPEORM_HOST=localhost
export TYPEORM_USERNAME=postgres
export TYPEORM_PASSWORD=postgres
export TYPEORM_DATABASE=lacpass_trusted_list_development
export TYPEORM_PORT=5432
export TYPEORM_SYNCHRONIZE=false
export TYPEORM_LOGGING=true
export TYPEORM_MIGRATIONS_RUN=true
export EXPOSED_CONTAINER_TYPEORM_PORT=5455

#REDIS

export REDIS_HOST=redis
export REDIS_PORT=6379
export REDIS_PASSWORD=redis
export EXPOSED_CONTAINER_REDIS_PORT=6405

#TOKEN

export JWT_SECRET=some-secret-string
export ACCESS_TOKEN_LIFE=360000000

#RATE LIMIT

export RATE_LIMIT_WINDOW=5
export RATE_LIMIT_MAX_REQUESTS=100

# custom variables
export TRUSTED_REGISTRIES="1,0xab7dd1Ca1Fb232b6E8bB5Bec1228892C7501b957,648540,0x048B946d673FA84b488601c7e2490085eFc61D0C,0x9e55c" # format: "INDEX_1,PD1,PD1_CID,COT1,COT1_CID--2,INDEX_2,PD2,PD2_CID-COT2,COT2_CID"
export TRUSTED_REGISTRIES_INDEX_PUBLIC_KEYS_TO_EXPOSE="1"
export DATABASE_URL="postgres://${TYPEORM_USERNAME}:${TYPEORM_PASSWORD}@${TYPEORM_HOST}:${EXPOSED_CONTAINER_TYPEORM_PORT}/${TYPEORM_DATABASE}" #default connection
export EXTERNAL_SOURCE_1="1,http://lacpass.create.cl:5001/trusted-parties"                                                                     # format: "INDEX_1,url_1--INDEX_2,url_2"
export RPC_CONNECTION_648540="http://35.185.112.219"
