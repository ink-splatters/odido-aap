#!/usr/bin/env zsh

set -e
set -o pipefail

if [[ ! "$(uname -a)" =~ ^Darwin.*arm64$ ]]; then
	cat <<'EOF'
ERROR: this script requires:
- macOS on Apple Silicon
- Odido iOS app is installed and authenticated
EOF
	exit 1
fi

echo 'Please wait...'

ODIDO_TOKEN=$(find ~/Library/Containers/**/nl.tmobile* \
	-name 'Cache.db' \
	-not -path '*crashlytics*' \
	-exec sqlite3 {} 'select proto_props from cfurl_cache_blob_data ;' \; |
	LC_ALL=C sed -E 's/.*([0-9a-f]{32}).*/\1/g' | tail -1)

if [[ $ODIDO_TOKEN =~ [0-9a-f]{32} ]]; then

	export ODIDO_TOKEN

	_shell=$(basename -- "$SHELL")
	_rc=.${_shell}rc
	if [ "$_shell" = "fish" ]; then
		_run="set -Ux ODIDO_TOKEN \$ODIDO_TOKEN"
	else
		_run="echo export ODIDO_TOKEN=\$ODIDO_TOKEN >> ~/$_rc"
	fi
	cat <<EOF
Done.

ODIDO_TOKEN environment variable was set
for persistence run:

$_run
EOF
else
	echo 'ERROR: bearer token was not found. Make sure Odido app is installed and you are logged in'
	exit 1
fi
