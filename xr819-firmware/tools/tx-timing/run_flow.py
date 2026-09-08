#!/usr/bin/env python3
"""Workstation receiver and bounded TCP trial. Invoke through the guarded harness."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import threading
import time
from typing import Any

from analyze_flow import analyze
from flow_format import CaptureError, END, SOCKET, START, decode

SSH = ['ssh', '-T', '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=yes',
       '-o', 'ConnectTimeout=4', '-o', 'ServerAliveInterval=5',
       '-o', 'ServerAliveCountMax=2', 'root@192.168.0.104']
ROOT = Path(__file__).resolve().parent


def private_open(path, mode='xb'):
    try:
        return open(path, mode, opener=lambda name, flags: os.open(name, flags, 0o600))
    except OSError as exc:
        raise CaptureError('private artifact creation failed') from exc


class Tape:
    def __init__(self, source, target, limit=512*1024*1024):
        self.source, self.target, self.limit = source, target, limit
        self.bytes = 0

    def read(self, size):
        raw = self.source.read(size)
        if self.bytes + len(raw) > self.limit:
            raise CaptureError('workstation byte cap')
        self.target.write(raw)
        self.bytes += len(raw)
        return raw


class Receiver:
    """Never queue events for the console: retain all bytes and just latest socket state."""
    def __init__(self, source, path):
        self.path = Path(path)
        self.ready, self.done = threading.Event(), threading.Event()
        self.lock = threading.Lock()
        self.socket_sequence = 0
        self.socket: dict[str, Any] | None = None
        self.start = self.end = None
        self.failure = None
        self.byte_count = 0
        self.source = source
        # Fail before starting traffic if the artifact cannot be created exclusively.
        self.target = private_open(self.path)
        self.thread = threading.Thread(target=self.pump, daemon=True)
        self.thread.start()

    def pump(self):
        try:
            with self.target:
                tape = Tape(self.source, self.target)
                for kind, obj in decode(tape):
                    if kind in (START, END, SOCKET):
                        assert isinstance(obj, dict)
                        received = time.monotonic_ns()
                        with self.lock:
                            if kind == START:
                                if type(obj.get('ready')) is not bool or not obj['ready']:
                                    raise CaptureError('collector not ready')
                                self.start = dict(received_ns=received, remote=obj)
                                self.ready.set()
                            elif kind == END:
                                self.end = dict(received_ns=received, remote=obj)
                            else:
                                self.socket = dict(received_ns=received, remote=obj)
                                self.socket_sequence += 1
                    # Flush in control intervals, not per event; final close flushes too.
                    if kind != 2:
                        self.target.flush()
                self.byte_count = tape.bytes
        except (CaptureError, OSError, ValueError) as error:
            self.failure = str(error) if isinstance(error, CaptureError) else type(error).__name__
        finally:
            self.done.set()

    def latest_socket(self):
        with self.lock:
            return self.socket_sequence, self.socket


class Progress:
    """Only consecutive, fresh single-socket samples can establish a progress gap."""
    def __init__(self, seconds):
        self.threshold = seconds * 10**9
        self.reset()

    def reset(self):
        self.last_ns = self.since = self.acked = None
        self.pending = False

    def stalled(self, sample):
        sockets = sample.get('sockets', [])
        if sample.get('error') or len(sockets) != 1 or 'bytes_acked' not in sockets[0]:
            self.reset()
            return False
        now = sample['after_ns']
        row = sockets[0]
        acked = row['bytes_acked']
        pending = row.get('notsent', 0) + row.get('unacked', 0) > 0
        if (self.last_ns is None or not 0 < now-self.last_ns <= 2500000000
                or not pending or not self.pending or acked != self.acked):
            self.since = now
        self.last_ns, self.acked, self.pending = now, acked, pending
        assert self.since is not None
        return pending and now-self.since >= self.threshold


def stop_child(child):
    if child is None or child.poll() is not None:
        return
    child.terminate()
    try:
        child.wait(timeout=5)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait(timeout=5)


def upload():
    subprocess.run(SSH + ['install -d -m 700 /root/xr819-full-flow'], check=True, timeout=15)
    paths = [ROOT / name for name in ('flow_format.py', 'collect_flow.py')]
    native = Path('/tmp/xr819-flow-native/project_flow-arm.so')
    if not native.is_file():
        raise CaptureError('build and test the native projector before deployment')
    subprocess.run(['scp', '-q', '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=yes',
                    '-o', 'ConnectTimeout=4', *map(str, paths),
                    'root@192.168.0.104:/root/xr819-full-flow/'], check=True, timeout=20)
    subprocess.run(['scp', '-q', '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=yes',
                    '-o', 'ConnectTimeout=4', str(native),
                    'root@192.168.0.104:/root/xr819-full-flow/project_flow.so'], check=True, timeout=20)
    import hashlib
    stamp = native.with_name('project_flow.source-sha256')
    if not stamp.is_file() or stamp.read_text().strip() != hashlib.sha256((ROOT / 'project_flow.c').read_bytes()).hexdigest():
        raise CaptureError('native projector source changed since qualification')
    checks = ''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  /root/xr819-full-flow/{p.name}\n'
                     for p in paths)
    checks += f'{hashlib.sha256(native.read_bytes()).hexdigest()}  /root/xr819-full-flow/project_flow.so\n'
    subprocess.run(SSH + ['chmod 600 /root/xr819-full-flow/*.py /root/xr819-full-flow/*.so; sha256sum -c -'],
                   input=checks, text=True, check=True, timeout=15)


def run(seconds, stall_seconds):
    path = Path(f'/tmp/xr819-full-flow-{time.time_ns()}.xtf')
    monitor = iperf = receiver = None
    result, capture_ok = 1, False
    manifest: dict[str, Any] = dict(seconds=seconds, stall_seconds=stall_seconds, capture=str(path))
    try:
        upload()
        manifest['launch_ns'] = time.monotonic_ns()
        monitor = subprocess.Popen(SSH + [
            f'python3 /root/xr819-full-flow/collect_flow.py --seconds {seconds+180}'],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, bufsize=0)
        assert monitor.stdout is not None
        receiver = Receiver(monitor.stdout, path)
        print(f'FULL_FLOW_FILE={path}', flush=True)
        if not receiver.ready.wait(timeout=15) or receiver.failure:
            raise CaptureError('collector readiness failed')
        print('TCP_MONITOR_READY', flush=True)
        iperf = subprocess.Popen(['iperf', '-c', '192.168.77.2', '-R', '-t', str(seconds), '-i', '5'])
        started, seen = time.monotonic_ns(), 0
        progress = Progress(stall_seconds)
        deadline = time.monotonic() + seconds + 30
        while iperf.poll() is None:
            if receiver.done.is_set() or receiver.failure:
                raise CaptureError('collector ended before traffic')
            if time.monotonic() >= deadline:
                raise CaptureError('traffic deadline')
            sequence, latest = receiver.latest_socket()
            age = time.monotonic_ns() - (latest['received_ns'] if latest else started)
            if age > 5000000000:
                raise CaptureError('socket sampler silence')
            if latest is not None and sequence != seen:
                if seen and sequence != seen+1:
                    # Full samples remain on disk; avoid inferring a stall across skipped samples.
                    progress.reset()
                seen = sequence
                print('FULL_FLOW_SOCKET ' + json.dumps(latest, sort_keys=True), flush=True)
                if progress.stalled(latest['remote']):
                    print('===TCP-PROGRESS-STALL-CAPTURE===', flush=True)
                    try:
                        failure = subprocess.run(['bash', '/tmp/xr819-capture-live-failure.sh'], timeout=150)
                        print(f'LIVE_CAPTURE_EXIT={failure.returncode}', flush=True)
                    except subprocess.TimeoutExpired:
                        print('LIVE_CAPTURE_TIMEOUT', flush=True)
                    result = 22
                    break
            time.sleep(0.05)
        else:
            result = iperf.returncode
    except (CaptureError, OSError, subprocess.SubprocessError) as error:
        print('FULL_FLOW_FAILURE=' + (str(error) if isinstance(error, CaptureError)
                                      else type(error).__name__), flush=True)
    finally:
        stop_child(iperf)
        if receiver is not None and monitor is not None:
            try:
                if monitor.poll() is None:
                    # Keep a small tail after traffic ends, then request explicit drain.
                    time.sleep(1)
                    manifest['stop_sent_ns'] = time.monotonic_ns()
                    assert monitor.stdin is not None
                    monitor.stdin.write(b'STOP\n')
                    monitor.stdin.flush()
                    monitor.stdin.close()
                if not receiver.done.wait(timeout=10):
                    raise CaptureError('receiver drain deadline')
                status = monitor.wait(timeout=5)
                capture_ok = status == 0 and receiver.failure is None and receiver.end is not None
                manifest.update(start=receiver.start, end=receiver.end, bytes=receiver.byte_count,
                                receiver_failure=receiver.failure, ssh_status=status)
            except (CaptureError, OSError, subprocess.SubprocessError) as error:
                manifest['shutdown_failure'] = type(error).__name__
            finally:
                stop_child(monitor)
                receiver.thread.join(timeout=5)
                manifest.update(start=receiver.start, end=receiver.end, bytes=receiver.byte_count,
                                receiver_failure=receiver.failure, ssh_status=monitor.poll())
                if monitor.stdin is not None:
                    monitor.stdin.close()
                if monitor.stdout is not None:
                    monitor.stdout.close()
        manifest.update(traffic_result=result, capture_ok=capture_ok)
        with private_open(str(path)+'.manifest.json', 'x') as output:
            json.dump(manifest, output, indent=2, sort_keys=True)
        print(f'FULL_FLOW_CAPTURE_OK={capture_ok} TRAFFIC_EXIT={result}', flush=True)
    if not capture_ok:
        return 24
    try:
        with path.open('rb') as source:
            report = analyze(source)
        with private_open(str(path)+'.report.json', 'x') as output:
            json.dump(report, output, indent=2, sort_keys=True)
        print('FULL_FLOW_REPORT=' + str(path)+'.report.json', flush=True)
    except (CaptureError, OSError) as error:
        print('FULL_FLOW_ANALYSIS_FAILURE=' + type(error).__name__, flush=True)
        return 25
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seconds', type=int, default=os.environ.get('XR819_TCP_SECONDS', '30'))
    parser.add_argument('--stall-seconds', type=int, default=os.environ.get('XR819_TCP_STALL_SECONDS', '10'))
    args = parser.parse_args()
    if not 1 <= args.seconds <= 600 or not 5 <= args.stall_seconds <= 60:
        parser.error('invalid duration or threshold')
    if os.environ.get('XR819_TCP_LOADED_SNAPSHOT_SECONDS', '0') != '0':
        parser.error('CPU snapshot mode is not supported by full capture')
    return run(args.seconds, args.stall_seconds)


if __name__ == '__main__':
    raise SystemExit(main())
