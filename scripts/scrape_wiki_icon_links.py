#!/usr/bin/env nix-shell
#! nix-shell -p python3 python3Packages.requests python3Packages.beautifulsoup4
#! nix-shell -i python

import requests
import argparse
from bs4 import BeautifulSoup
import sys
import json

BASE_URL = "https://wiki.bloodontheclocktower.com/"
DEFAULT_URLS = [
    BASE_URL + url
    for url in [
        "Trouble_Brewing",
        "Sects_%26_Violets",
        "Bad_Moon_Rising",
        "Travellers",
        "Fabled",
        "Experimental",
    ]
]

p = argparse.ArgumentParser()
p.add_argument(
    "--url",
    help="Override the BotC Wiki URLs to scrape",
    nargs="*",
    default=DEFAULT_URLS,
)
p.add_argument(
    "--merge-into", help="Merge into existing JSON file", type=argparse.FileType("r")
)
p.add_argument(
    "--merge-path", help="Dot-separated path to merge into in the existing JSON file"
)
args = p.parse_args()
urls: list[str] = args.url
merge_path: list[str] = args.merge_path.split(".") if args.merge_path else []

objs = []

for url in urls:
    print(f"> Downloading {url}", file=sys.stderr)
    r = requests.get(url)
    r.raise_for_status()
    print("> Done!", file=sys.stderr)

    soup = BeautifulSoup(r.text, "html.parser")
    icons: list[BeautifulSoup] = soup.find_all("img", class_="thumbimage")

    for i in icons:
        name = i.find_parent("a").get("title")
        icon = i.get("src")
        if not icon.startswith("http"):
            icon = BASE_URL + icon
        obj = {"name": name, "icon": icon}
        objs.append(obj)

if args.merge_into:
    data = json.load(args.merge_into)
    existing_list = data
    for key in merge_path:
        existing_list = existing_list[key]
    for obj in objs:
        for existing in existing_list:
            if existing["name"] == obj["name"]:
                print(f"> Merging {obj['name']}", file=sys.stderr)
                existing.update(obj)
                break
        else:
            print(f"> Skipping {obj['name']}", file=sys.stderr)
    print(json.dumps(data, indent=4))
else:
    print(json.dumps(objs, indent=4))
