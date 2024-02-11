#!/usr/bin/env python
import requests
import json
from datetime import datetime
import json
import logging
import colorlog
import os

# Credits
# [Romkabouter430](https://tweakers.net/gallery/2749)

# get token on Apple Silicon, given Odido app is installed via App Store:
# fd nl.tmobile.mytmobile "$HOME"/Library/Containers | rg 'Library/Caches' | xargs -n1 -I% sqlite3 -json '%/Cache.db' 'select * from cfurl_cache_blob_data'  | jq  '.[].proto_props | select (. != null)' | rg -o 'Bearer ([0-9a-f]{32})' --replace '$1' | tail -1
if __name__ == "__main__":

    import http.client as http_client

    http_client.HTTPConnection.debuglevel = 1

    handler = colorlog.StreamHandler()
    handler.setFormatter(
        colorlog.ColoredFormatter("%(log_color)s[%(levelname)s] %(message)s")
    )

    log = colorlog.getLogger()
    log.setLevel(logging.DEBUG)
    log.addHandler(handler)

    if "ODIDO_TOKEN" not in os.environ:
        log.fatal("ODIDO_TOKEN env var is required")

    accesstoken = os.environ["ODIDO_TOKEN"]

    # Create new header with Authorization
    headers = {
        "Authorization": "Bearer " + accesstoken,
        "User-Agent": "T-Mobile 5.3.28 (Android 10; 10)",
        "Accept": "application/json",
    }
    response = requests.get(
        "https://capi.t-mobile.nl/account/current?resourcelabel=LinkedSubscriptions",
        headers=headers,
    )
    dict = json.loads(response.content)

    # call the Resources Url
    response = requests.get(dict["Resources"][0]["Url"], headers=headers)
    dict = json.loads(response.content)

    subscriptionUrl = dict["subscriptions"][0]["SubscriptionURL"]
    response = requests.get(subscriptionUrl + "/roamingbundles", headers=headers)
    dict = json.loads(response.content)

    data = {"Bundles": [{"BuyingCode": "A0DAY01"}]}

    totalRemaining = 0

    for bundle in dict["Bundles"]:
        if bundle["ZoneColor"] == "NL":
            remaining = bundle["Remaining"]
            totalRemaining += remaining["Value"]

    if round(totalRemaining / 1024, 0) < 1500:
        self.interval = 600

    if round(totalRemaining / 1024, 0) < 1000:
        post_resp = requests.post(
            subscriptionUrl + "/roamingbundles", json=data, headers=headers
        )
        log.debug(post_resp)
        log.info("2000MB aangevuld")
        self.interval = int(self.args["interval"])
    else:
        log.info(
            "There is "
            + str(round(totalRemaining / 1024, 0))
            + " MB remaining, no need to update"
        )
