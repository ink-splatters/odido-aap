# odido aap

odido.nl aanvullers automated

## Installation

assuming Python is installed

```sh
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

## Usage

```sh
source .venv/bin/activate
./odido.py
```

**NOTE:** you may want the script to be run by `cron`. In this case, you will also want ODIDO_THRESHOLD env variable
(in megabytes) to be set to some 300-350. 

Otherwise the API will return an error: it's only allowed to activate the next aanvuller when
around ~350mb is left from the previous one. 

## Getting Bearer Token

**NOTE**: this is only supported on Apple Silicon and requires authenticated Odido iOS app.
Consult [tweakers.net](https://tweakers.net) for more methods to obtain the token

```sh
./get_bearer_apple_silicon.sh
```

Enjoy!

## Credits

[Romkabouter430](https://tweakers.net/gallery/2749)

