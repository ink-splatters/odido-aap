#!/usr/bin/env python
import http.client
import json
import os
import sys
from typing import Any, Literal, overload

import requests
from dotenv import load_dotenv
from loguru import logger
from requests import Response

load_dotenv()

# Credits
# [Romkabouter430](https://tweakers.net/gallery/2749)
# @Lyceris-chan (refactoring)


def get_required_var(name: str) -> str:
    if name not in os.environ:
        logger.error(f"required environment variable: {name} is not set")
        exit(1)
    return os.environ[name]


user_id: str = get_required_var("ODIDO_USER_ID")
subscription_url: str = os.environ.get("ODIDO_CACHED_SUBSCRIPTION_URL", "")
access_token: str = get_required_var("ODIDO_TOKEN")
threshold: int = int(os.environ.get("ODIDO_THRESHOLD", 400))
debug = int(os.environ.get("ODIDO_DEBUG", 0))
force_topup = int(os.environ.get("ODIDO_FORCE_TOPUP", 0))


def setup_logger() -> None:
    logger.remove()
    logger.add(
        sys.stderr,
        level="DEBUG" if debug else "INFO",
        # backtrace=True,
        # diagnose=True
    )

    # wire-level logging
    # WARNING: this exposes your credentials in plaintext
    if debug != 0:
        http.client.HTTPConnection.debuglevel = 1


def check_and_update_data() -> None:
    """
    Checks the Odido data balance and tops it up if below the threshold.
    """

    @overload
    def verify_get_response_data(
        response: Response, require_json: Literal[True]
    ) -> dict[str, Any]: ...

    @overload
    def verify_get_response_data(
        response: Response, require_json: Literal[False] = False
    ) -> dict[str, Any] | str: ...

    def verify_get_response_data(
        response: Response, require_json: bool = False
    ) -> dict[str, Any] | str:
        """Helper function to check HTTP response status."""
        code = response.status_code
        if not response.ok:
            logger.error(f"Request failed: {code} {response.reason}")
            # Raise an exception instead of exiting to allow the main loop to continue
            response.raise_for_status()

        try:
            data = response.json()
            logger.debug(f"Response: {json.dumps(data, indent=4)}")
        except ValueError:
            data = response.text
            if require_json:
                raise ValueError(f"Payload is not a json: {data}.") from None
            logger.debug(f"Response: {data}")
        return data

    # Create new header with Authorization
    headers = {
        "Authorization": f"Bearer {access_token}",
        "User-Agent": "T-Mobile 5.3.28 (Android 10; 10)",
        "Accept": "application/json",
    }

    global subscription_url
    if subscription_url:
        logger.info("Using passed Subscription URL.")
    else:
        if force_topup:
            logger.warning(
                "Do cache subscription or force_topup does not make sense, as optimization!"
            )
        logger.info("Fetching subscription details...")
        response = requests.get(
            f"https://capi.odido.nl/{user_id}/linkedsubscriptions",
            headers=headers,
        )

        data = verify_get_response_data(response, require_json=True)
        subscription_url = data["subscriptions"][0]["SubscriptionURL"]

    if not force_topup:
        logger.info("Fetching roaming bundle information...")
        response = requests.get(subscription_url + "/roamingbundles", headers=headers)
        data = verify_get_response_data(response, require_json=True)

        total_remaining = 0
        for bundle in data["Bundles"]:
            if bundle["ZoneColor"] == "NL":
                remaining = bundle["Remaining"]
                total_remaining += remaining["Value"]

        total_remaining_mb = round(total_remaining / 1024, 0)
        logger.info(f"Data remaining: {total_remaining_mb} MB (Threshold: {threshold} MB)")

    if force_topup or total_remaining_mb < threshold:
        if force_topup:
            logger.info("Attempting to force top-up...")
        else:
            logger.warning("Data is below the threshold. Attempting to top up...")
        data = {"Bundles": [{"BuyingCode": "A0DAY01"}]}
        response = requests.post(subscription_url + "/roamingbundles", json=data, headers=headers)
        data = verify_get_response_data(response)
        logger.info("Successfully requested 2000MB.")

    else:
        logger.info("Data is sufficient. No action needed.")


if __name__ == "__main__":
    try:
        setup_logger()

        check_and_update_data()
    except Exception:
        logger.opt(exception=True).error("Error:")
