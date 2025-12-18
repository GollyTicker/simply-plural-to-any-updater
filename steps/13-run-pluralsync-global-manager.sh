#/bin/bash

set -euo pipefail

echo "$GLOBAL_PLURALSYNC_SIMPLY_PLURAL_READ_WRITE_ADMIN_TOKEN" > /dev/null

cargo run --bin pluralsync-global-manager
