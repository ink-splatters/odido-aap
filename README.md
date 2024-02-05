# Monkey Buiseness

odido.nl aanvullers automated


## Installation

assuming Python is installed


```shell
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Getting Token

```shell
brew install jq ripgrep
```

given the app is installed via AppStore:

```shell
 fd nl.tmobile.mytmobile "$HOME"/Library/Containers | \
    rg 'Library/Caches' | \
    xargs -n1 -I% sqlite3 -json '%/Cache.db' 'select * from cfurl_cache_blob_data' | \
    jq  '.[].proto_props | select (. != null)' | \
    rg -o 'Bearer ([0-9a-f]{32})' --replace '$1' | \
    tail -1
 ```

expose it via `ODIDO_TOKEN` env var

Enjoy!

## Credits
# [Romkabouter430](https://tweakers.net/gallery/2749)
