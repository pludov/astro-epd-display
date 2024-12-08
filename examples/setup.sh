#!/bin/bash

set -x

curl -X POST -H 'Content-Type: application/json' --data-binary '@state.json' http://127.0.0.1:3000/state
curl -X POST -H 'Content-Type: application/json' --data-binary '@template.yaml' http://127.0.0.1:3000/template

curl  -f --show-error  http://127.0.0.1:3000/display | jq .