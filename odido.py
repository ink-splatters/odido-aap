#!/usr/bin/env python
import json
import os
import sys

import requests
from loguru import logger

# Credits
# [Romkabouter430](https://tweakers.net/gallery/2749)
# @Lyceris-chan (refactoring)


def setup_logger():
    logger.remove()
    logger.add(
        sys.stderr,
        level="INFO",  # increase verbosity: level="DEBUG",
        backtrace=True,
        diagnose=False,
    )

    # wire-level logging
    # http.client.HTTPConnection.debuglevel = 1


def check_and_update_data(accesstoken, threshold):
    """
    Checks the Odido data balance and tops it up if below the threshold.
    """

    def check_and_get_response_data(response):
        """Helper function to check HTTP response status."""
        code = response.status_code
        if not response.ok:
            reason = response.reason
            logger.error(f"Request failed: {code} {reason}")
            # Raise an exception instead of exiting to allow the main loop to continue
            response.raise_for_status()

        data = response.json()
        logger.debug(f"response: {json.dumps(data, indent=4)}")
        return data

    # Create new header with Authorization
    headers = {
        "Authorization": "Bearer " + accesstoken,
        "User-Agent": "T-Mobile 5.3.28 (Android 10; 10)",
        "Accept": "application/json",
    }

    logger.info("Fetching subscription details...")
    response = requests.get(
        f"https://capi.odido.nl/{user_id}/linkedsubscriptions",
        headers=headers,
    )

    data = check_and_get_response_data(response)
    subscription_url = data["subscriptions"][0]["SubscriptionURL"]

    logger.info("Fetching roaming bundle information...")
    response = requests.get(subscription_url + "/roamingbundles", headers=headers)
    data = check_and_get_response_data(response)

    total_remaining = 0
    for bundle in data["Bundles"]:
        if bundle["ZoneColor"] == "NL":
            remaining = bundle["Remaining"]
            total_remaining += remaining["Value"]

    total_remaining_mb = round(total_remaining / 1024, 0)
    logger.info(f"Data remaining: {total_remaining_mb} MB (Threshold: {threshold} MB)")

    if total_remaining_mb < threshold:
        logger.warning("Data is below threshold. Attempting to top up...")
        data = {"Bundles": [{"BuyingCode": "A0DAY01"}]}
        response = requests.post(subscription_url + "/roamingbundles", json=data, headers=headers)
        data = check_and_get_response_data(response)
        logger.info("Successfully requested 2000MB top-up.")

    else:
        logger.info("Data is sufficient. No action needed.")


def get_required_var(name: str) -> str:
    if name not in os.environ:
        logger.fatal(f"required environment variable: {name} is not set")
        exit(1)
    return os.environ[name]


if __name__ == "__main__":
    try:
        setup_logger()

        user_id = get_required_var("ODIDO_USER_ID")
        accesstoken = get_required_var("ODIDO_TOKEN")

        threshold = int(os.environ.get("ODIDO_THRESHOLD", 1500))

        check_and_update_data(accesstoken, threshold)
    except Exception:
        logger.opt(exception=True).debug("Debug error:")
