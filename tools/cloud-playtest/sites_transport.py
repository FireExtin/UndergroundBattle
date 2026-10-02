"""Safe diagnostics and same-origin redirect policy for private Sites QA.

The environment proxy CA is trusted by Playwright's request client. A redirect
fulfilled into Chromium otherwise leaves that client and its trusted CA chain.
Keep only same-origin redirects inside the request client; never log headers.
"""
import time
from urllib.parse import urljoin, urlsplit


def safe_path(url):
    parts = urlsplit(url)
    return parts.netloc + parts.path


def same_origin_redirect(response, current_url, origin):
    if response.status not in (301, 302, 303, 307, 308):
        return None
    location = response.headers.get('location')
    if not location:
        return None
    destination = urljoin(current_url, location)
    parts = urlsplit(destination)
    if parts.scheme == 'https' and parts.netloc == origin and not parts.username and not parts.password:
        return destination
    return None


def transport_failure(error, url):
    # Raw Playwright errors can echo secret request headers. Retain only a
    # whitelist of network cause names, never the raw exception or query string.
    message = str(error).lower()
    cause = next((cause for cause in (
        'ERR_CERT_AUTHORITY_INVALID', 'ECONNRESET', 'ENETUNREACH', 'ETIMEDOUT',
        'Timeout', 'aborted', 'socket hang up', 'unable to verify', 'self-signed',
    ) if cause.lower() in message), 'other-suppressed')
    return {'kind': 'qa-route-failure', 'time': time.time(), 'path': safe_path(url),
            'errorType': type(error).__name__, 'cause': cause, 'syntheticHttpStatus': 503}
