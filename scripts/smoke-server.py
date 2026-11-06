"""Exercise the real loopback server without opening a browser.

Usage: python scripts/smoke-server.py target/release/monitorawake-server.exe
Creates an isolated temporary profile and never touches the normal user profile.
"""

import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.parse
import urllib.request


def start(binary, profile):
    process = subprocess.Popen(
        [str(binary)],
        env={**os.environ, 'MONITORAWAKE_DATA_DIR': str(profile)},
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding='utf-8',
        creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
    )
    lines = queue.Queue()
    threading.Thread(target=lambda: [lines.put(line) for line in process.stdout], daemon=True).start()
    try:
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            try:
                line = lines.get(timeout=0.2)
            except queue.Empty:
                if process.poll() is not None:
                    raise RuntimeError(process.stderr.read())
                continue
            if line.startswith('Configurazione: '):
                url = urllib.parse.urlsplit(line.removeprefix('Configurazione: ').strip())
                token = urllib.parse.parse_qs(url.fragment)['token'][0]
                return process, f'{url.scheme}://{url.netloc}', token
        raise RuntimeError('Server startup timed out')
    except BaseException:
        process.terminate()
        process.wait(timeout=5)
        raise


def request(base, token, path='/api/state', action=None, extra=None):
    headers = {'Authorization': f'Bearer {token}'} if token else {}
    if action is not None:
        headers['Content-Type'] = 'application/json'
    headers.update(extra or {})
    req = urllib.request.Request(base + path, headers=headers,
        data=json.dumps(action).encode() if action is not None else None)
    # Ignore machine HTTP proxy settings for this strictly loopback test.
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    try:
        with opener.open(req, timeout=5) as response:
            data = response.read()
            return response.status, json.loads(data) if 'json' in response.headers.get('Content-Type', '') else data
    except urllib.error.HTTPError as error:
        return error.code, error.read()


def until(base, token, predicate):
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        code, snapshot = request(base, token)
        assert code == 200
        if predicate(snapshot):
            return snapshot
        time.sleep(0.1)
    raise AssertionError('Expected state was not reached')


def main():
    binary = Path(sys.argv[1]).resolve(strict=True)
    profile = Path(tempfile.mkdtemp(prefix='monitorawake-smoke-'))
    process, base, token = start(binary, profile)
    try:
        assert request(base, '', '/')[0] == 200
        assert request(base, '')[0] == 401
        assert request(base, token, extra={'Host': 'untrusted.example'})[0] == 403
        assert request(base, token, extra={'Origin': 'https://untrusted.example'})[0] == 403
        invalid = {'type': 'add_timer', 'name': 'Invalid', 'seconds': 0, 'keep_awake': False}
        assert request(base, token, '/api/action', invalid)[0] == 400
        action = {'type': 'command', 'text': 'timer prova 2 secondi'}
        assert request(base, '', '/api/action', action)[0] == 401
        assert request(base, token, '/api/action', action)[0] == 200
        expired = until(base, token, lambda s: s['timers'][0]['status'] == 'expired')
        timer_id = expired['timers'][0]['id']
        ack = {'type': 'acknowledge_timer', 'id': timer_id}
        assert request(base, token, '/api/action', ack)[0] == 200
        if expired['capabilities']['system']:
            assert request(base, token, '/api/action', {'type': 'set_awake', 'mode': 'system'})[0] == 200
            until(base, token, lambda s: s['applied_mode'] == 'system')
            assert request(base, token, '/api/action', {'type': 'set_awake', 'mode': 'off'})[0] == 200
            until(base, token, lambda s: s['applied_mode'] == 'off')
        duplicate = subprocess.run([str(binary)], env={**os.environ, 'MONITORAWAKE_DATA_DIR': str(profile)},
            capture_output=True, timeout=5, creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0)
        assert duplicate.returncode != 0
    finally:
        process.terminate()
        process.wait(timeout=5)
    process, base, next_token = start(binary, profile)
    try:
        assert next_token != token
        assert request(base, token)[0] == 401
        code, restored = request(base, next_token)
        assert code == 200 and restored['timers'][0]['id'] == timer_id
        assert restored['timers'][0]['status'] == 'acknowledged'
        assert restored['manual_mode'] == 'off'
    finally:
        process.terminate()
        process.wait(timeout=5)
    print('PASS: HTTP auth/origin/host, embedded assets, timer expiry, acknowledgement, native awake on/off, single instance, restart and token rotation.')
    print(f'Isolated test profile retained: {profile}')


if __name__ == '__main__':
    main()
