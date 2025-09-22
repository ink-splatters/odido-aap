# odido aap

odido.nl aanvullers automated

## Setup

- python3.9+
- uv (recommended)
- direnv

### Initializing env

```sh
uv venv
uv pip install -r requirements.txt
direnv allow .
```

### Obtaining Credentials

#### Apple Silicon

1. Install Odido official app from AppStore
1. Authenticate
1. Run

```sh
./get_creds_from_cache_db.sh
```

It will create .env file for you with variables neded for the script to work. if you installed direnv and followed
[Initializing Env](#initializing-env) you are all set.

#### From iMazing Backup

If the previous method is unaccessible to you, you can obtain credentials from iMazing backup of your iPhone or iPad.

1. Locate Odido.nl app in iMazing
1. Export it (in the form of `.imazingapp`), rename to .zip, extract all the data and locate `Cache.db`. If `Cache.db-wal` file is present, it must remain.
1. Set `CACHE_PATH` env var to the correct PATH with `Cache.db`
1. If all set correctly, you will get the same result - credentials written in `.env`

## Usage

Make sure you followed [Initializing env](#initializing-env). Among other things, direnv should have activated python virtual env
for you.

Now, run:

```sh
./odido.py
```

**NOTE:** you may want the script to be run by `cron`. In this case, you will also want ODIDO_THRESHOLD env variable
(in megabytes) to be set to some 300-350.

Otherwise the API will return an error: it's only allowed to activate the next aanvuller when
around ~350mb is left from the previous one.

Enjoy!

## Credits

[Romkabouter430](https://tweakers.net/gallery/2749)
