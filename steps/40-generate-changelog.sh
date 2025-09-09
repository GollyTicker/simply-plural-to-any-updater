#!/bin/bash

set -euo pipefail

CURRENT_TAG="$1"
CHANGELOG_FILE="$2"

echo "Current tag: $CURRENT_TAG"


# Get all tags sorted by committerdate (most recent first)
SORTED_TAGS=($(git tag --sort=-committerdate))
PREVIOUS_TAG=""


# Find the current tag in the sorted list and get the one before it (which is chronologically previous)
for i in "${!SORTED_TAGS[@]}"; do
    if [[ "${SORTED_TAGS[$i]}" == "$CURRENT_TAG" ]]; then
        if [[ $((i + 1)) -lt ${#SORTED_TAGS[@]} ]]; then
            PREVIOUS_TAG="${SORTED_TAGS[$((i + 1))]}"
            break
        fi
    fi
done


echo "Generating changelog from $PREVIOUS_TAG to $CURRENT_TAG..."
COMMIT_LOG=$(git log --pretty=format:"* %s (%h)" $PREVIOUS_TAG..$CURRENT_TAG)
echo -e "## Changelog ($PREVIOUS_TAG → $CURRENT_TAG)\n\n$COMMIT_LOG" > $CHANGELOG_FILE

cat $CHANGELOG_FILE
