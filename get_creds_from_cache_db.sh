#!/usr/bin/env bash

set -e
set -o pipefail

DEFAULT_CACHE_DIR=(~/Library/Containers/*/Data/Library/Caches/nl.tmobile.mytmobile)
CACHE_DIR="${CACHE_DIR:-${DEFAULT_CACHE_DIR[0]}}"

# Check if .env exists and ask for confirmation
if [ -f .env ]; then
	read -p ".env file exists. Overwrite? (y/N): " -n 1 -r
	echo
	if [[ ! $REPLY =~ ^[Yy]$ ]]; then
		echo "Aborted."
		exit 0
	fi
else
	# Copy from .env.example if .env doesn't exist
	if [ -f .env.example ]; then
		cp .env.example .env
	else
		echo "ERROR: .env.example not found"
		exit 1
	fi
fi

# Extract bearer token
BEARER=$(/usr/bin/find "$CACHE_DIR" -name 'Cache.db' \
	-exec /usr/bin/sqlite3 {} 'select proto_props from cfurl_cache_blob_data ;' \; |
	LC_ALL=C sed -E 's/.*([0-9a-f]{32}).*/\1/g' | tail -1 | tr -d '\n')

# Extract ID from request_object URLs
USER_ID=$(/usr/bin/find "$CACHE_DIR" -name 'Cache.db' \
	-exec /usr/bin/sqlite3 {} 'select request_object from cfurl_cache_blob_data ;' \; |
	strings | grep -o 'capi\.odido\.nl/[0-9a-f]\{12\}' | sed 's/.*\///' | head -1 | tr -d '\n')

# Substitute values in .env
if [[ "$OSTYPE" == "darwin"* ]]; then
	# macOS sed requires backup extension
	sed -i '' "s/^ODIDO_USER_ID=.*/ODIDO_USER_ID=$USER_ID/" .env
	sed -i '' "s/^ODIDO_TOKEN=.*/ODIDO_TOKEN=$BEARER/" .env
else
	sed -i "s/^ODIDO_USER_ID=.*/ODIDO_USER_ID=$USER_ID/" .env
	sed -i "s/^ODIDO_TOKEN=.*/ODIDO_TOKEN=$BEARER/" .env
fi

echo "Credentials written to .env"
