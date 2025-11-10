# odido aap

odido.nl aanvullers automated

## Setup

TODO

### Obtaining Credentials

#### Apple Silicon

1. Install Odido official app from AppStore
1. Authenticate
1. Run

```sh
./get_creds_from_cache_db.sh
```

It will create .env file for you with variables needed for the script to work.

#### From iMazing Backup

If the previous method is inaccessible to you, you can obtain credentials from iMazing backup of your iPhone or iPad.

1. Locate Odido.nl app in iMazing
1. Export it (in the form of `.imazingapp`), rename to .zip, extract all the data and locate `Cache.db`. If `Cache.db-wal` file is present, it must remain.
1. Set `CACHE_DIR` env var to the directory containing `Cache.db`
1. If all set correctly, you will get the same result - credentials written in `.env`

## Usage

TODO

## Credits

[Romkabouter430](https://tweakers.net/gallery/2749)
