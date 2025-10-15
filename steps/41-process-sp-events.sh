#!/bin/bash

set -euo pipefail

cat sp-events.log | \
    jq -R 'capture("^\\[(?<timestamp>[^ ]+).*?\\((?<user_id>.*)\\).*?: \\+(?<changed>.*) (?<json>.*)$") | .json |= fromjson'

