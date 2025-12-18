#!/bin/bash

set -euo pipefail

ssh ayake.net "docker logs pluralsync-api-public-test 2>&1 | grep 'SP WS payload' | grep -v 'Authentication violation' | grep -v \")': {}\" | grep -v 'Successfully authenticated'" >> sp-events.log

