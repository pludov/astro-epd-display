#!/bin/bash

PORT=${PORT:-3000}
set -x

curl -X POST -H 'Content-Type: application/json' --data-binary '@state.json' http://127.0.0.1:$PORT/state
curl -X POST -H 'Content-Type: application/json' --data-binary '@template.yaml' http://127.0.0.1:$PORT/template

