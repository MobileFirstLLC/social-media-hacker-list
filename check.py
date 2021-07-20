"""check all urls in readme for ~OK (acceptable) response"""
import re
import sys
from requests import request

PATTERN = r'(https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9][a-zA-Z0-9-]+[a-z' \
          r'A-Z0-9]\.[^\s)\']{2,}|www\.[a-zA-Z0-9][a-zA-Z0-9-]+[a-zA-' \
          r'Z0-9]\.[^\s]{2,}|https?:\/\/(?:www\.|(?!www))[a-zA-Z0-9]+' \
          r'\.[^\s)\']{2,}|www\.[a-zA-Z0-9]+\.[^\s)\']{2,})'


def is_ok(status_code, domain):
    return status_code in [200, 403, 406] or \
           ('reddit.com' in domain and status_code == 502)


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


def print_failures(failures):
    output = '\n'.join([f'- {msg}: {url}' for (msg, url) in failures])
    print(f'{len(failures)} failure(s):\n{output}')


def main():
    urls = list(set(re.findall(PATTERN, open("README.md", "r").read())))
    total, failures, progress = len(urls), [], -1

    print(f'Checking {total} urls')
    for index, url in enumerate(urls):
        success, code = try_request(url, "HEAD")
        if not success:
            failures.append((code, url))
        progress = display_progress(index, progress, total)
    if failures:
        print_failures(failures)
        sys.exit(1)
    print(f'no issues')


if __name__ == '__main__':
    main()
