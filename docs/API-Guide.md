# API Guide

## Requisites before continuing

TODO

## Interacting via Terminal

1. Set API url to interact with

```sh
api_url=http://localhost:3025 # Set LACPass API url
```

2. Verify certificate

```shell
path_to_qr=../demo-preconectaton/qrs/base45-lacchain  # you can obtain the data from a file
base45_health_certificate=`cat $path_to_qr` # or just paste the base45 data here
# process
verify_url="$api_url"/api/v1/certificates/verify-b45
curl -X 'POST' ${verify_url} -H 'accept: */*' -H 'Content-Type: text/plain' -d ${base45_health_certificate}
```

## Swagger

Visit `http://localhost:3025/swagger-ui/index.html`, make sure to point to the right port and host
