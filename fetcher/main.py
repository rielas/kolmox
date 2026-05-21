from selenium import webdriver
from selenium.webdriver.support.ui import WebDriverWait
from selenium.common.exceptions import TimeoutException
import os
import csv
import argparse
from path import get_path


""" Fetches all provided URLs and saves their HTML content to files."""


def wait_for_real_content(driver, timeout: int = 30):
    # IMDB (and other sites behind AWS WAF) first serve a JS challenge page
    # with an empty <title> that reloads itself once the challenge passes.
    # Waiting for readyState=complete AND a non-empty title catches the
    # reload to the real page.
    try:
        WebDriverWait(driver, timeout).until(
            lambda d: (
                d.execute_script("return document.readyState") == "complete"
                and d.title.strip() != ""
            )
        )
    except TimeoutException:
        pass


def main(path: str):
    with open(path + "dataset.csv", "r") as file:
        reader = csv.DictReader(file)
        entries = list(reader)

    driver = webdriver.Chrome()
    try:
        for entry in entries:
            url = entry["url"]
            filepath = path + "/" + get_path(url)
            os.makedirs(os.path.dirname(filepath), exist_ok=True)

            if os.path.exists(filepath):
                continue

            driver.get(url)
            wait_for_real_content(driver)

            with open(filepath, "w") as f:
                f.write(driver.page_source)
    finally:
        driver.quit()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fetch URLs and save HTML content")
    parser.add_argument(
        "path", help="Path to dataset directory (e.g., dataset/wikipedia/)"
    )

    args = parser.parse_args()
    main(args.path)
