"""check all urls in readme for ~OK (acceptable) response"""
import re
import sys
import json
from requests import request
from urllib.parse import urlparse
from dateutil import parser
from datetime import datetime

GH_TOKEN = sys.argv[1]
PATTERN = r'(https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9][a-zA-Z0-9-]+[a-z' \
          r'A-Z0-9]\.[^\s)\']{2,}|www\.[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-' \
          r'Z0-9]\.[^\s]{2,}|https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9]+' \
          r'\.[^\s)\']{2,}|www\.[a-zA-Z0-9]+\.[^\s)\']{2,})'


def is_gh_repo(url):
    return 'github.com' in urlparse(url).netloc and len(urlparse(url).path.split('/')) > 2


def is_ok(status_code, domain):
    return status_code in [200, 403, 406] or \
           ('reddit.com' in urlparse(domain).netloc and status_code == 502)


def display_progress(n, progress, total):
    percent = (n * 100) // total
    if percent % 10 == 0 and percent > progress:
        print(f'...{percent} %')
        return percent
    return progress


def try_request(url, method, retry=0):
    # noinspection PyBroadException
    try:
        response = request(method, url, headers={'Accept': '*/*'})
        code = response.status_code
        success = is_ok(code, url)
        return (success, code) if success or retry > 0 \
            else try_request(url, "GET", 1)
    except Exception as e:
        return False, f'error: ' + str(e)


def active_repo(url):
    try:
        [owner, repo] = urlparse(url).path.split('/')[1:3]
        api_url = f'https://api.github.com/repos/{owner}/{repo}'
        response = request('GET', api_url, headers={
            'Authorization': f'token {GH_TOKEN}',
            'Accept': 'application/vnd.github.v3+json'})
        if is_ok(response.status_code, url):
            last_update = parser.parse(json.loads(response.content)['updated_at'])
            days_since_last_update = (datetime.now() - last_update.replace(tzinfo=None)).days
            return (True, "OK") if days_since_last_update <= 365 else (False, "INACTIVE")
        return False, response.status_code
    except Exception as e:
        return False, f'error: ' + str(e)


def print_failures(failures):
    output = '\n'.join([f'- {msg}: {url}' for (msg, url) in failures])
    print(f'{len(failures)} failure(s):\n{output}')


def main():
    urls = list(set(re.findall(PATTERN, open("README.md", "r").read())))
    total, failures, progress = len(urls), [], -1

    print(f'Checking {total} urls')
    for index, url in enumerate(urls):
        success, code = active_repo(url) if is_gh_repo(url) else try_request(url, "HEAD")
        if not success:
            failures.append((code, url))
        progress = display_progress(index, progress, total)
    if failures:
        print_failures(failures)
        sys.exit(1)
    print(f'no issues')


if __name__ == '__main__':
    main()
