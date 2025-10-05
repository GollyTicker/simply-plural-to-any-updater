#/bin/bash

set -euo pipefail

echo "$GLOBAL_SP2ANY_SIMPLY_PLURAL_READ_WRITE_ADMIN_TOKEN" > /dev/null

cargo run --bin sp2any-global-manager
