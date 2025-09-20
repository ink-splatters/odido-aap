#!/usr/bin/env python
import http.client as http_client
import json
import logging
import os

import colorlog
import requests

# Credits
# [Romkabouter430](https://tweakers.net/gallery/2749)

if __name__ == "__main__":

    def check(response):
        code = response.status_code
        reason = response.reason
        if code not in [200, 202]:
            log.fatal(f"{code}: {reason}")
            exit(1)

    http_client.HTTPConnection.debuglevel = 1

    handler = colorlog.StreamHandler()
    handler.setFormatter(colorlog.ColoredFormatter("%(log_color)s[%(levelname)s] %(message)s"))

    log = colorlog.getLogger()
    log.setLevel(logging.DEBUG)
    log.addHandler(handler)

    if "ODIDO_TOKEN" not in os.environ:
        log.fatal("ODIDO_TOKEN environment variable is not set")
        exit(1)

    threshold = int(os.environ.get("ODIDO_THRESHOLD", 1500))

    accesstoken = os.environ["ODIDO_TOKEN"]

    # Create new header with Authorization
    headers = {
        "Authorization": "Bearer " + accesstoken,
        "User-Agent": "T-Mobile 5.3.28 (Android 10; 10)",
        "Accept": "application/json",
    }
    response = requests.get(
        "https://capi.odido.nl/c88084b603f5/linkedsubscriptions",
        headers=headers,
    )
    check(response)

    dict = json.loads(response.content)

    subscription_url = dict["subscriptions"][0]["SubscriptionURL"]
    response = requests.get(subscription_url + "/roamingbundles", headers=headers)
    check(response)

    dict = json.loads(response.content)

    data = {"Bundles": [{"BuyingCode": "A0DAY01"}]}

    total_remaining = 0

    for bundle in dict["Bundles"]:
        if bundle["ZoneColor"] == "NL":
            remaining = bundle["Remaining"]
            total_remaining += remaining["Value"]

    log.info(f"threshold: {threshold}")
    if round(total_remaining / 1024, 0) < threshold:
        response = requests.post(subscription_url + "/roamingbundles", json=data, headers=headers)
        check(response)
        log.debug(response)
        log.info("2000MB aangevuld")
        # self.interval = int(self.args["interval"])
    else:
        log.info(
            "There is " + str(round(total_remaining / 1024, 0)) + " MB remaining, no need to update"
        )
