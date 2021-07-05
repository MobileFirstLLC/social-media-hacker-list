"""check all urls in readme for ~OK (acceptable) response"""
import re
import sys
import requests

README = "README.md"
URL_PATTERN = r'(https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-Z0-9]\.[^\s)\']{2,}|www\.[a-zA-Z0-9][' \
              r'a-zA-Z0-9-]+[a-zA-Z0-9]\.[^\s]{2,}|https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9]+\.[^\s)\']{2,' \
              r'}|www\.[a-zA-Z0-9]+\.[^\s)\']{2,})'

acceptable_headers = [200, 403, 406]
readme_text = open(README, "r").read()
urls = list(set(re.findall(URL_PATTERN, readme_text)))
total = len(urls)
tenth = total // 10
failures = []

def false_alert(status_code, url):
    return status_code in acceptable_headers or \
        # sometimes reddit throws 502 for bots -> bypass
        'reddit' in url and status_code == 502

print(f'Checking {total} urls...')
for index, url in enumerate(urls):
    # noinspection PyBroadException
    try:
        # request headers only
        response = requests.head(
            url, headers={'Accept': '*/*'}, timeout=30.0)
        if not false_alert(response.status_code, url):
            # sometimes head request fails - try get
            response = requests.get(
                url, headers={'Accept': '*/*'}, timeout=30.0)
            # when everything fails, register as failure
            if not false_alert(response.status_code, url):
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
