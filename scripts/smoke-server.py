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
            if line.startswith('Settings: '):
                url = urllib.parse.urlsplit(line.removeprefix('Settings: ').strip())
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
    legacy = json.dumps({'schema_version': 1, 'timers': [], 'settings': {'animations': False, 'notifications': False}}).encode()
    (profile / 'state.json').write_bytes(legacy)
    process, base, token = start(binary, profile)
    try:
        assert request(base, token)[1]['schema_version'] == 4
        backups = list(profile.glob('state.json.backup-v1-*.json'))
        assert len(backups) == 1 and backups[0].read_bytes() == legacy
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
        code, created = request(base, token, '/api/action', {'type': 'create_note'})
        assert code == 200
        note_id = created['state']['notes'][0]['id']
        body = '\u03b1' * 9000  # UTF-8 and JSON escaping exceed the old 16 KiB action limit.
        update = {'type': 'update_note', 'id': note_id, 'expected_revision': 0, 'title': 'Persistent note', 'body': body}
        assert request(base, token, '/api/action', update)[0] == 200
        assert request(base, token, '/api/action', update)[0] == 400  # Stale editor cannot overwrite.
        assert request(base, token, '/api/action', {**update, 'expected_revision': 1, 'body': 'x' * 10001})[0] == 400
        assert request(base, token, '/api/action', {'type': 'set_note_visible', 'id': note_id, 'visible': False})[0] == 200
        assert request(base, token, '/api/action', {'type': 'set_note_appearance', 'id': note_id, 'color': 'blue', 'pinned': True})[0] == 200
        assert request(base, token, '/api/action', {'type': 'set_note_reminder', 'id': note_id, 'seconds': 60})[0] == 200
        _, current = request(base, token)
        settings = {**current['settings'], 'phrases': ['One small step at a time.'], 'speech_enabled': True}
        assert request(base, token, '/api/action', {'type': 'update_settings', 'settings': settings})[0] == 200
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
        assert restored['notes'][0]['id'] == note_id and restored['notes'][0]['body'] == body
        assert restored['notes'][0]['visible'] is False and restored['notes'][0]['pinned'] is True
        assert restored['notes'][0]['color'] == 'blue' and restored['settings']['phrases'] == settings['phrases']
        assert len(list(profile.glob('state.json.backup-v1-*.json'))) == 1
        linked_timer = restored['notes'][0]['reminder_id']
        assert request(base, next_token, '/api/action', {'type': 'delete_note', 'id': note_id})[0] == 200
        _, deleted = request(base, next_token)
        assert not deleted['notes']
        assert next(timer for timer in deleted['timers'] if timer['id'] == linked_timer)['status'] == 'cancelled'
    finally:
        process.terminate()
        process.wait(timeout=5)
    print('PASS: HTTP auth/origin/host, embedded assets, timer expiry, native awake on/off, single instance, schema migration and exact backup, Unicode notes, stale/invalid edits, hide/pin, note reminder deletion, recurring phrases, restart and token rotation.')
    print(f'Isolated test profile retained: {profile}')


if __name__ == '__main__':
    main()
