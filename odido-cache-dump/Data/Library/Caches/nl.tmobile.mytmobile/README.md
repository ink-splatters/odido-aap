# odido.nl app cache

this is encrypted dump of the cache not intended for public eyes

## Obtaining your own cache

Only Apple Silicon is supported as you will need to run iOS app on your Mac

1. Insall Odido app from Dutch AppStore (other AppStore locations may work)
2. Locate the db at:

```
/Users/<username>/Library/Containers/<UUID>/Data/Library/Caches/nl.tmobile.mytmobile/Cache.db
```

3. Optionally, if you are planning to transfer it to your location, run `VACCUUM ;` on it in order to flush
sqlite3 WAL, so that you would take only the single file with you.

## Extracting bearer token

Only Apple Silicon is supported as you will need to run iOS app on your Mac. 

1. Make sure you have installed Odido iOS app and authenticated
2. Use `./get_bearer_apple_silicon.sh` in the root of the repo
