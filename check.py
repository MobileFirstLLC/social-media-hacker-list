"""check all urls in readme for ~OK (acceptable) response"""

import re
import sys
from json import loads
from requests import request
from urllib.parse import urlparse
from dateutil.parser import parse
from datetime import datetime

TOKEN = sys.argv[1] if len(sys.argv) > 1 else None
LIMIT = int(sys.argv[2]) if len(sys.argv) > 2 else None
PATTERN = r'(https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9][a-zA-Z0-9-]+[a-z' \
          r'A-Z0-9]\.[^\s)\']{2,}|www\.[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-' \
          r'Z0-9]\.[^\s]{2,}|https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9]+' \
          r'\.[^\s)\']{2,}|www\.[a-zA-Z0-9]+\.[^\s)\']{2,})'


def path(url):
    return urlparse(url).path.split('/')


def host(url, domain):
    return domain in urlparse(url).netloc


def ok(status, url):
    return status in [200, 403, 406] or \
           (status == 502 and host(url, 'reddit.com'))


def active(latest_change):
    delta = datetime.now() - latest_change.replace(tzinfo=None)
    return delta.days <= 365


def check_url(url, method, retry=0):
    response = request(method, url, headers={'Accept': '*/*'})
    success = ok(response.status_code, url)
    if success or retry > 0:
        return success, response.status_code
    return check_url(url, "GET", 1)


def active_repo(url):
    [owner, repo] = path(url)[1:3]
    api_url = f'https://api.github.com/repos/{owner}/{repo}'
    headers = {'Accept': 'application/vnd.github.v3+json'}
    if TOKEN:
        headers['Authorization'] = f'token {TOKEN}'
    response = request('GET', api_url, headers=headers)
    if not ok(response.status_code, url):
        return False, response.status_code
    if not active(parse(loads(response.content)['updated_at'])):
        return False, "INACTIVE"
    return True, 200


def main():
    readme = open("README.md", "r").read()
    urls = list(set(re.findall(PATTERN, readme)))[0:LIMIT]
    fails, total, progress = [], len(urls), 0

    print(f'Checking {total} entries...')
    for index, url in enumerate(urls):
        is_repo = host(url, 'github.com') and len(path(url)) > 2
        try:
            success, code = active_repo(url) if is_repo \
                else check_url(url, "HEAD")
            if not success:
                fails.append((code, url))
        except Exception as e:
            fails.append((f'error: {e}', url))
        percent = (index * 100) // total
        if percent % 10 == 0 and percent > progress:
            print(f'...{percent} % ({len(fails)})')
            progress = percent
    if fails:
        output = '\n'.join([f'- {m}: {u}' for m, u in fails])
        print(f'{len(fails)} failure(s):\n{output}')
        sys.exit(1)
    print(f'no issues')


if __name__ == '__main__':
    main()
