#!/usr/bin/env python3
"""Board-side full numeric event projection. Stdout is binary, stdin accepts STOP."""
import argparse
import collections
import ctypes
import json
import multiprocessing
import os
from pathlib import Path
import re
import resource
import select
import signal
import subprocess
import sys
import time

from flow_format import (CaptureError, DATA, END, HEALTH, MAX_PAYLOAD, RECORD,
                         SOCKET, START, control, data_frame, frame, project)

FLOW = Path('/sys/kernel/tracing/instances/xr819_static_flow')
SCALARS = ('bytes_acked', 'bytes_sent', 'bytes_retrans', 'notsent', 'unacked',
           'cwnd', 'ssthresh', 'data_segs_out', 'data_segs_in', 'lost', 'sacked',
           'delivered', 'dsack_dups', 'snd_wnd', 'backoff')


def number(text):
    if len(text) > 20:
        raise CaptureError('numeric text cap')
    try:
        return int(text)
    except ValueError as exc:
        raise CaptureError('invalid numeric text') from exc


def ring_stats(flow=FLOW):
    result = []
    for path in sorted(flow.glob('per_cpu/cpu*/stats')):
        values = {}
        for line in path.read_text().splitlines():
            name, sep, value = line.partition(':')
            if sep and name in ('overrun', 'commit overrun', 'dropped events', 'entries', 'read events'):
                values[name.replace(' ', '_')] = number(value.strip())
        if not {'overrun', 'commit_overrun', 'dropped_events'} <= values.keys():
            raise CaptureError('incomplete ring statistics')
        result.append(dict(cpu=number(path.parent.name[3:]), **values))
    if not result:
        raise CaptureError('missing ring statistics')
    return result


def has_loss(stats):
    return any(row[key] for row in stats for key in ('overrun', 'commit_overrun', 'dropped_events'))


def project_sockets(text):
    sockets = []
    for line in text.splitlines():
        if not re.search(r'\bbytes_acked:\d+', line):
            continue
        row = {key: number(match[1]) for key in SCALARS
               if (match := re.search(r'\b' + key + r':(\d+)', line))}
        if match := re.search(r'\bretrans:(\d+)/(\d+)', line):
            row.update(retrans_pending=number(match[1]), retrans_total=number(match[2]))
        for key in ('rtt', 'rto', 'minrtt'):
            if match := re.search(r'\b' + key + r':(\d+)(?:\.(\d+))?', line):
                row[key + '_us'] = number(match[1]) * 1000 + number((match[2] or '').ljust(3, '0')[:3])
        sockets.append(row)
        if len(sockets) > 8:
            raise CaptureError('socket count cap')
    return sockets


def socket_snapshot():
    """Bounded subprocess output; no raw ss text is transmitted or logged."""
    before = time.monotonic_ns()
    command = ['ip', 'netns', 'exec', 'wifi-test', 'ss', '-tin', 'state', 'established',
               '( sport = :5001 and dst = 192.168.77.1 )']
    child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    assert child.stdout is not None
    output = bytearray()
    deadline = time.monotonic() + 3
    try:
        os.set_blocking(child.stdout.fileno(), False)
        while True:
            if time.monotonic() >= deadline:
                raise CaptureError('socket sampler timeout')
            if select.select([child.stdout], [], [], 0.05)[0]:
                chunk = os.read(child.stdout.fileno(), 4096)
                if not chunk:
                    break
                output.extend(chunk)
                if len(output) > 65536:
                    raise CaptureError('socket output cap')
        if child.wait(timeout=max(0.01, deadline-time.monotonic())):
            raise CaptureError('socket sampler exit')
        return dict(before_ns=before, after_ns=time.monotonic_ns(),
                    sockets=project_sockets(output.decode('ascii', 'replace')), error=0)
    finally:
        if child.poll() is None:
            child.kill()
        child.wait(timeout=3)
        child.stdout.close()


def sample_worker(connection, stop):
    next_ring = 0
    try:
        while not stop.is_set():
            started = time.monotonic()
            try:
                sample = socket_snapshot()
            except (CaptureError, OSError, subprocess.TimeoutExpired):
                sample = dict(before_ns=time.monotonic_ns(), after_ns=time.monotonic_ns(),
                              sockets=[], error=1)
            usage = resource.getrusage(resource.RUSAGE_SELF)
            sample['sampler_cpu_us'] = int((usage.ru_utime + usage.ru_stime) * 10**6)
            connection.send_bytes(json.dumps([SOCKET, sample]).encode('ascii'))
            if time.monotonic() >= next_ring:
                connection.send_bytes(json.dumps([HEALTH, dict(mono_ns=time.monotonic_ns(),
                                                              rings=ring_stats())]).encode('ascii'))
                next_ring = time.monotonic() + 5
            stop.wait(max(0, 1 - (time.monotonic()-started)))
    except (BrokenPipeError, EOFError, OSError, CaptureError):
        # Parent detects worker exit and fails capture; never print into binary stdout.
        return
    finally:
        connection.close()


class Output:
    """Bounded output queue. A blocked consumer fails rather than delaying trace reads."""
    def __init__(self, fd, max_bytes=512*1024*1024, max_pending=1024*1024):
        self.fd, self.max_bytes, self.max_pending = fd, max_bytes, max_pending
        self.frames = collections.deque()
        self.pending = self.bytes = 0
        os.set_blocking(fd, False)

    def put(self, frame, final=False):
        reserve = 0 if final else MAX_PAYLOAD + 32
        if self.bytes + len(frame) + reserve > self.max_bytes:
            raise CaptureError('capture byte cap')
        if self.pending + len(frame) > self.max_pending:
            raise CaptureError('output backpressure cap')
        self.frames.append(memoryview(frame))
        self.pending += len(frame)
        self.bytes += len(frame)
        self.flush()

    def flush(self):
        while self.frames:
            try:
                size = os.write(self.fd, self.frames[0])
            except BlockingIOError:
                return
            if not size:
                raise CaptureError('zero output write')
            self.pending -= size
            front = self.frames.popleft()
            if size < len(front):
                self.frames.appendleft(front[size:])

    def finish(self, timeout: float = 3):
        deadline = time.monotonic() + timeout
        while self.frames:
            self.flush()
            if self.frames:
                remaining = deadline-time.monotonic()
                if remaining <= 0:
                    raise CaptureError('output final drain timeout')
                select.select([], [self.fd], [], min(remaining, 0.05))


class Trace:
    def __init__(self, flow=FLOW):
        self.flow, self.owned, self.fd = flow, False, None

    def setup(self):
        root = self.flow.parent.parent
        if (self.flow.parent / 'xr819_flow').exists():
            raise CaptureError('rejected trace instance present')
        if 'xr819_flow' in (root / 'kprobe_events').read_text():
            raise CaptureError('rejected dynamic probes present')
        self.flow.mkdir()
        self.owned = True
        self.set('tracing_on', '0')
        self.set('trace_clock', 'mono')
        self.set('buffer_size_kb', '256')
        self.set('events/mmc/mmc_request_done/filter',
                 'cmd_opcode == 53 && (cmd_err != 0 || data_err != 0 || stop_err != 0)')
        self.set('events/net/net_dev_queue/filter', 'name == "wlan0"')
        for event in ('cw1200_queued', 'cw1200_transport', 'cw1200_confirmed'):
            if not (self.flow / f'events/cw1200_flow/{event}/format').is_file():
                raise CaptureError('missing static event')
        self.fd = os.open(self.flow / 'trace_pipe', os.O_RDONLY | os.O_NONBLOCK)

    def set(self, path, value):
        (self.flow / path).write_text(value)

    def enable(self):
        for path in ('cw1200_flow', 'net/net_dev_queue', 'mmc/mmc_request_done'):
            self.set('events/' + path + '/enable', '1')
        self.set('tracing_on', '1')

    def disable(self):
        if self.owned:
            self.set('tracing_on', '0')
            self.set('events/enable', '0')

    def close(self):
        try:
            self.disable()
        finally:
            if self.fd is not None:
                os.close(self.fd)
                self.fd = None
            if self.owned:
                self.flow.rmdir()
                self.owned = False


class Projector:
    def flush_input(self):
        """Reference projector has no batch staging."""

    def __init__(self, output):
        self.output, self.pending, self.total = output, b'', 0
        self.counts = collections.Counter()

    def feed(self, data):
        parts = (self.pending + data).split(b'\n')
        self.pending = parts.pop()
        if len(self.pending) > 8192 or any(len(line) > 8192 for line in parts):
            raise CaptureError('trace line cap')
        batch = []
        for line in parts:
            row = project(line.decode('ascii', 'strict'), self.total+1)
            if row is not None:
                batch.append(row)
                self.total += 1
                self.counts[row.kind] += 1
            if len(batch) == 256:
                self.output.put(data_frame(batch))
                batch = []
        if batch:
            self.output.put(data_frame(batch))


class NativeProjector:
    """Batch parser: the Python reference is for fixtures, not hardware throughput."""
    def __init__(self, output, library=None):
        self.output, self.pending, self.total = output, b'', 0
        self.counts = collections.Counter()
        self.input = bytearray()
        self.batches = 0
        self.library = ctypes.CDLL(str(library or Path(__file__).with_name('project_flow.so')))
        self.library.flow_project_abi.restype = ctypes.c_uint
        if self.library.flow_project_abi() != 1:
            raise CaptureError('native projector ABI mismatch')
        self.library.flow_project.argtypes = [ctypes.c_char_p, ctypes.c_size_t, ctypes.c_uint64,
                                              ctypes.c_void_p, ctypes.c_size_t,
                                              ctypes.POINTER(ctypes.c_size_t)]
        self.library.flow_project.restype = ctypes.c_int
        self.buffer = ctypes.create_string_buffer(262144)

    def feed(self, data):
        self.input.extend(data)
        if len(self.input) >= 16384:
            self.flush_input()

    def flush_input(self):
        if not self.input:
            return
        data = bytes(self.input)
        self.input.clear()
        self.batches += 1
        self.project_batch(data)

    def project_batch(self, data):
        pending = self.pending + data
        while True:
            end = pending.rfind(b'\n', 0, 65536) + 1
            if not end:
                break
            records = ctypes.c_size_t()
            status = self.library.flow_project(pending[:end], end, self.total+1,
                                                self.buffer, len(self.buffer), ctypes.byref(records))
            if records.value > len(self.buffer)//RECORD.size:
                raise CaptureError('native projector output bounds')
            raw = ctypes.string_at(self.buffer, records.value*RECORD.size)
            # Preserve the successfully parsed prefix even when the next line fails.
            for offset in range(0, len(raw), MAX_PAYLOAD):
                part = raw[offset:offset+MAX_PAYLOAD]
                count = len(part)//RECORD.size
                self.output.put(frame(DATA, self.total+1, count, part))
                self.total += count
                self.counts.update(memoryview(part)[18::64])
            pending = pending[end:]
            if status:
                self.pending = b''
                raise CaptureError('native projection rejected input: code ' + str(status))
        self.pending = pending
        if len(self.pending) > 8192:
            raise CaptureError('trace line cap')


def collect(seconds, stdin=0, stdout=1, use_native=True):
    trace, output = Trace(), Output(stdout)
    projector = NativeProjector(output) if use_native else Projector(output)
    receive, send = multiprocessing.Pipe(duplex=False)
    stop = multiprocessing.Event()
    worker = multiprocessing.Process(target=sample_worker, args=(send, stop), daemon=True)
    interrupted = []
    for sig in (signal.SIGTERM, signal.SIGHUP, signal.SIGINT):
        signal.signal(sig, lambda number, _frame: interrupted.__setitem__(slice(None), [number]))
    clean, started, reason = False, False, 'setup'
    trace_reads = empty_reads = input_bytes = 0
    try:
        trace.setup()
        initial = ring_stats()
        if has_loss(initial):
            raise CaptureError('nonzero initial ring loss')
        worker.start()
        send.close()
        trace.enable()
        before = time.time_ns()
        mono = time.monotonic_ns()
        after = time.time_ns()
        output.put(control(START, 0, dict(record_bytes=RECORD.size, ready=True,
                                         wall_before_ns=before, mono_ns=mono,
                                         wall_after_ns=after, rings=initial)))
        started = True
        os.set_blocking(stdin, False)
        command = b''
        deadline = time.monotonic()+seconds
        next_batch = time.monotonic() + 0.01
        while True:
            if interrupted or time.monotonic() >= deadline:
                raise CaptureError('signal or capture deadline')
            if not worker.is_alive():
                raise CaptureError('sampler exited')
            assert trace.fd is not None
            wait = max(0, min(0.01, next_batch-time.monotonic()))
            ready, _, _ = select.select([trace.fd, stdin, receive.fileno()], [], [], wait)
            if trace.fd in ready:
                trace_reads += 1
                try:
                    data = os.read(trace.fd, 65536)
                except BlockingIOError:
                    data = b''
                input_bytes += len(data)
                if data:
                    projector.feed(data)
                else:
                    empty_reads += 1
                    # Some trace-pipe poll modes remain readable at an empty watermark.
                    # Bound that spin without delaying reads when data is available.
                    time.sleep(0.001)
            if receive.fileno() in ready:
                projector.flush_input()
                kind, obj = json.loads(receive.recv_bytes(MAX_PAYLOAD))
                if (kind == SOCKET and obj['error']) or (kind == HEALTH and has_loss(obj['rings'])):
                    raise CaptureError('sampler failure or kernel ring loss')
                output.put(control(kind, projector.total, obj))
            if stdin in ready:
                part = os.read(stdin, 32)
                if not part:
                    raise CaptureError('control channel EOF without STOP')
                command += part
                if command == b'STOP\n':
                    break
                if len(command) >= 5 or not b'STOP\n'.startswith(command):
                    raise CaptureError('invalid stop command')
            if time.monotonic() >= next_batch:
                projector.flush_input()
                next_batch = time.monotonic() + 0.01
            output.flush()
        trace.disable()
        # Allow already-entered tracepoint writers to commit after disable.
        drain_deadline = time.monotonic() + 2
        quiet_since = time.monotonic()
        while time.monotonic() - quiet_since < 0.1:
            if time.monotonic() >= drain_deadline:
                raise CaptureError('trace did not quiesce')
            try:
                data = os.read(trace.fd, 65536)
            except BlockingIOError:
                data = b''
            if data:
                projector.feed(data)
                quiet_since = time.monotonic()
            else:
                time.sleep(0.01)
        projector.flush_input()
        final = ring_stats()
        if projector.pending or has_loss(final) or any(row.get('entries', 0) for row in final):
            raise CaptureError('partial event or final ring loss')
        clean, reason = True, 'stopped'
    except (CaptureError, OSError, ValueError, EOFError) as error:
        reason = str(error) if isinstance(error, CaptureError) else type(error).__name__
    finally:
        stop.set()
        if worker.pid is not None:
            worker.join(timeout=0.2)
            if worker.is_alive():
                worker.terminate()
                worker.join(timeout=1)
        receive.close()
        send.close()
        try:
            trace.disable()
            final = ring_stats() if trace.owned else []
            usage = resource.getrusage(resource.RUSAGE_SELF)
            if started:
                output.put(control(END, projector.total, dict(
                    events=projector.total, event_counts=dict(projector.counts), clean=clean,
                    reason=reason, partial_bytes=len(projector.pending), rings=final,
                    mono_ns=time.monotonic_ns(),
                    collector_cpu_us=int((usage.ru_utime+usage.ru_stime)*10**6),
                    collector_user_us=int(usage.ru_utime*10**6),
                    collector_system_us=int(usage.ru_stime*10**6),
                    trace_reads=trace_reads, empty_reads=empty_reads,
                    input_bytes=input_bytes,
                    projection_batches=getattr(projector, 'batches', 0))), final=True)
                output.finish()
        except (CaptureError, OSError):
            clean = False
        finally:
            trace.close()
    return 0 if clean else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--seconds', type=int, default=780, choices=range(1, 901))
    args = parser.parse_args()
    if os.uname().release != '6.18.0-xr819-test+' or os.geteuid() != 0:
        parser.exit(1, 'Requires the qualified board kernel and root.\n')
    return collect(args.seconds)


if __name__ == '__main__':
    raise SystemExit(main())
