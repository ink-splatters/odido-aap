#!/usr/bin/env python
import http.client as http_client
import json
import logging
import os

import colorlog
import requests
from dotenv import load_dotenv

load_dotenv()


def check(response):
    code = response.status_code
    reason = response.reason
    if code not in (200, 202):
        log.fatal(f"{code}: {reason}")
        exit(1)


def get_required_var(name: str) -> str:
    if name not in os.environ:
        log.fatal(f"{name} environment variable is not set")
        exit(1)
    return os.environ[name]


# Credits
# [Romkabouter430](https://tweakers.net/gallery/2749)

if __name__ == "__main__":
    http_client.HTTPConnection.debuglevel = 1

    handler = colorlog.StreamHandler()
    handler.setFormatter(colorlog.ColoredFormatter("%(log_color)s[%(levelname)s] %(message)s"))

    log = colorlog.getLogger()
    log.setLevel(logging.DEBUG)
    log.addHandler(handler)

    user_id = get_required_var("ODIDO_USER_ID")
    accesstoken = get_required_var("ODIDO_TOKEN")

    threshold = int(os.environ.get("ODIDO_THRESHOLD", 1500))

    headers = {
        "Authorization": f"Bearer {accesstoken}",
        "User-Agent": "T-Mobile 5.3.28 (Android 10; 10)",
        "Accept": "application/json",
    }
    response = requests.get(
        f"https://capi.odido.nl/{user_id}/linkedsubscriptions",
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
