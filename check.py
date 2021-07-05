"""check all urls in readme for OK response code"""
import re
import sys
import requests

README = "README.md"
URL_PATTERN = r'(https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-Z0-9]\.[^\s)\']{2,}|www\.[a-zA-Z0-9][' \
              r'a-zA-Z0-9-]+[a-zA-Z0-9]\.[^\s]{2,}|https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9]+\.[^\s)\']{2,' \
              r'}|www\.[a-zA-Z0-9]+\.[^\s)\']{2,})'
acceptable_headers = [200, 403, 406]

# read the readme
readme_text = open(README, "r").read()
urls = list(set(re.findall(URL_PATTERN, readme_text)))
total = len(urls)
tenth = total // 10
failures = []

# check all urls
print(f'checking {total} urls...')
for index, url in enumerate(urls):
    # noinspection PyBroadException
    try:
        response = requests.head(
            url, headers={'Accept': '*/*'}, timeout=30.0)
        # sometimes head request fails
        if response.status_code not in acceptable_headers:
            response = requests.get(
                url, headers={'Accept': '*/*'}, timeout=30.0)
            # if get fails also register it as failure
            if response.status_code not in acceptable_headers:
                failures.append((response.status_code, url))
    except Exception as e:
        failures.append(('error', url))
    if index > 0 and index % tenth == 0:
        print(f'...{((index * 100) // total)} %')

if not failures:
    print(f'no issues')
    sys.exit(0)

print(f'{len(failures)} failure(s):')
for (msg, url) in failures:
    print(f'- {msg}: {url}')
sys.exit(1)
