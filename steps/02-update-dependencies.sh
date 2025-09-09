#!/bin/bash

set -euo pipefail

cargo update

(cd frontend && npm install && ncu -u)
(cd bridge-frontend && npm install && ncu -u)
