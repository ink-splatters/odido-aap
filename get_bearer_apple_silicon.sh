#!/usr/bin/env bash

set -e
set -o pipefail

_uname=$(uname)
_arch=$(arch)

if [[ $_uname != "Darwin" || $_arch != "arm64" ]]; then
	cat <<'EOF'
ERROR: this script requires macOS on Apple Silicon
EOF
	exit 1
fi

if [[ "$1" = "" ]] || [[ "$1" =~ -h|--help ]]; then
	cat <<EOF
Dumps odido.nl bearer token from Odido official app cache data

Usage:
	$0 <[-h|--help] | [print]>

	-h|--help	prints this help message
	print		dumps bearer token

CAUTION: the token will be printed to stdout in plaintext!

EOF
	exit 1
fi

if [ "$1" != "print" ]; then
	echo "ERROR: command is invalid: $1"
	exit 1
fi

/usr/bin/find ~/Library/Containers/*/Data/Library/Caches/nl.tmobile.mytmobile -name 'Cache.db' \
	-exec /usr/bin/sqlite3 {} 'select proto_props from cfurl_cache_blob_data ;' \; |
	LC_ALL=C sed -E 's/.*([0-9a-f]{32}).*/\1/g' | tail -1 | tr -d '\n'
