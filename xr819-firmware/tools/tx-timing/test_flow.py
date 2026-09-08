#!/usr/bin/env python3
"""Protocol and timing fixtures. No board access or raw capture artifacts."""
import collections
import io
import os
import random
import tempfile
from unittest import mock
import struct
import unittest

from analyze_flow import Matcher, analyze
from flow_format import (CaptureError, DATA, END, RECORD, START, Event,
                         control, data_frame, decode, project)
from collect_flow import NativeProjector, Output, Projector, collect, project_sockets
from run_flow import Progress, Receiver, Tape

RINGS = [dict(cpu=0, entries=0, overrun=0, commit_overrun=0, dropped_events=0)]


def event(sequence, ns, kind, **fields):
    from flow_format import SCHEMAS
    return Event(sequence, ns, 0, kind, tuple(fields[k] for k in SCHEMAS[kind][1]))


def episode(first=1, base=0, cookie=7, status=0, flags=0, requeue=0, previous=7):
    return [
        event(first, base+1, 1, id=cookie, previous=previous, len=1576, queue=2, requeue=requeue),
        event(first+1, base+11, 2, id=cookie, msgid=4, len=2048, data=1, done=0, result=0),
        event(first+2, base+31, 2, id=cookie, msgid=4, len=2048, data=1, done=1, result=0),
        event(first+3, base+61, 3, id=cookie, status=status, rate=19, ack=0,
              flags=flags, media=999999, queued=888888),
    ]


def capture(events):
    counts = dict(collections.Counter(str(e.kind) for e in events))
    return (control(START, 0, {'record_bytes': RECORD.size, 'rings': RINGS, 'ready': True}) + data_frame(events)
            + control(END, len(events), {'events': len(events), 'clean': True,
                                        'rings': RINGS, 'partial_bytes': 0, 'event_counts': counts}))


class FormatTests(unittest.TestCase):
    def test_projection_exact_timestamp_and_redaction(self):
        row = project('kworker-12 [000] ... 184.000001234: cw1200_queued: '
                      'id=7 previous=7 len=1576 queue=2 requeue=0 secret=1234 ptr=0xdeadbeef', 1)
        self.assertIsNotNone(row)
        assert row is not None
        self.assertEqual(row.ns, 184000001234)
        self.assertEqual(row.fields()['len'], 1576)
        self.assertEqual(len(row.pack()), 64)
        self.assertNotIn(b'secret', row.pack())
        self.assertNotIn(struct.pack('<I', 0xdeadbeef), row.pack())

    def test_deployed_mmc_completion_has_no_cmd_arg(self):
        row = project('irq-1 [001] ... 7.500000: mmc_request_done: '
                      'mmc1: end struct mmc_request[deadbeef]: cmd_opcode=53 '
                      'cmd_err=-110 cmd_resp=0xa 0xb 0xc 0xd cmd_retries=0 '
                      'stop_opcode=0 stop_err=0 stop_resp=0x0 0x0 0x0 0x0 '
                      'stop_retries=0 sbc_opcode=0 sbc_err=0 sbc_resp=0x0 0x0 0x0 0x0 '
                      'sbc_retries=0 bytes_xfered=0 data_err=-110 tag=-1 can_retune=0', 1)
        assert row is not None
        self.assertEqual(row.fields()['data_err'], -110)
        self.assertNotIn('cmd_resp', row.fields())
        self.assertNotIn('write', row.fields())
        self.assertEqual(Event.unpack(row.pack()), row)

    def test_netdev_ignores_pointer(self):
        row = project('x [000] ... 1.0: net_dev_queue: dev=wlan0 skbaddr=deadbeef len=1514', 1)
        assert row is not None
        self.assertEqual(row.values, (1514,))

    def test_malformed_duplicate_loss_and_range(self):
        lines = ['CPU:0 [LOST 1 EVENTS]', 'x [000] ... 1.0: cw1200_queued: id=1',
                 'x [000] ... 1.0: net_dev_queue: len=1 len=2',
                 'x [000] ... 1.0: net_dev_queue: len=-1']
        for line in lines:
            with self.subTest(line=line), self.assertRaises(CaptureError):
                project(line, 1)

    def test_roundtrip_and_short_reads(self):
        class ShortReader(io.BytesIO):
            def read(self, size: int | None = -1):
                return super().read(3 if size is None or size < 0 else min(size, 3))
        rows = episode()
        decoded = [r for kind, r in decode(ShortReader(capture(rows))) if kind == DATA]
        self.assertEqual(rows, decoded)

    def test_truncation_corruption_sequence_and_trailing_data(self):
        raw = capture(episode())
        bad = bytearray(raw)
        bad[80] ^= 1
        for body in (raw[:-1], raw[:80], bytes(bad), raw+b'x', b''):
            with self.subTest(size=len(body)), self.assertRaises(CaptureError):
                list(decode(io.BytesIO(body)))
        wrong_start = control(START, 0, {'record_bytes': 64, 'rings': RINGS}) + data_frame(episode(first=2))
        with self.assertRaises(CaptureError):
            list(decode(io.BytesIO(wrong_start)))

    def test_failed_footer(self):
        raw = control(START, 0, {'record_bytes': 64, 'rings': RINGS}) + control(END, 0, {'events': 0, 'clean': False})
        with self.assertRaises(CaptureError):
            list(decode(io.BytesIO(raw)))


class MatcherTests(unittest.TestCase):
    def test_full_population_and_cookie_reuse(self):
        rows = [row for i in range(200) for row in episode(first=i*4+1, base=i*100)]
        result = analyze(io.BytesIO(capture(rows)))
        self.assertEqual(result['counts']['matched_episodes'], 200)
        self.assertEqual(result['censored_live_episodes'], 0)
        for metric, duration in [('admission_to_write_start', 10), ('write_call', 20),
                                 ('write_return_to_confirmation', 30), ('admission_to_confirmation', 60)]:
            self.assertEqual(result['metrics'][metric]['sum_ns'], duration*200)
            lo, hi = result['metrics'][metric]['p99_bounds_ns']
            self.assertLessEqual(lo, duration)
            self.assertGreaterEqual(hi, duration)

    def test_requeue_links_old_and_new(self):
        rows = episode(status=11, flags=2) + episode(first=5, base=100, cookie=8, requeue=1)
        result = analyze(io.BytesIO(capture(rows)))
        self.assertEqual(result['counts']['linked_requeues'], 1)
        self.assertEqual(result['unlinked_requeue_confirmations'], 0)

    def test_overlap_is_not_clamped(self):
        rows = episode()
        rows[2] = event(3, 21, 3, id=7, status=0, rate=19, ack=0, flags=0, media=0, queued=0)
        rows[3] = event(4, 31, 2, id=7, msgid=4, len=2048, data=1, done=1, result=0)
        result = analyze(io.BytesIO(capture(rows)))
        self.assertEqual(result['metrics']['response_overlap']['sum_ns'], 10)
        self.assertNotIn('write_return_to_confirmation', result['metrics'])

    def test_incomplete_is_censored_not_completed(self):
        result = analyze(io.BytesIO(capture(episode()[:2])))
        self.assertEqual(result['censored_live_episodes'], 1)
        self.assertEqual(result['metrics'], {})

    def test_collision_and_memory_cap(self):
        matcher = Matcher(max_live=1)
        matcher.accept(episode()[0])
        with self.assertRaises(CaptureError):
            matcher.accept(episode(first=2, base=10)[0])
        with self.assertRaises(CaptureError):
            matcher.accept(episode(first=3, base=20, cookie=8)[0])

    def test_cpu_regression(self):
        matcher = Matcher()
        matcher.accept(episode(base=100)[0])
        with self.assertRaises(CaptureError):
            matcher.accept(episode()[1])


def trace_lines(rows):
    from flow_format import SCHEMAS
    return b''.join((f'x [{r.cpu:03d}] ... {r.ns // 10**9}.{r.ns % 10**9:09d}: '
                     + SCHEMAS[r.kind][0] + ': '
                     + ' '.join(f'{k}={v}' for k, v in r.fields().items()) + '\n').encode('ascii')
                    for r in rows)


class CollectorTests(unittest.TestCase):
    def test_all_events_survive_split_reads_and_batching(self):
        rows = [r for i in range(1000) for r in episode(i*4+1, i*100)]
        raw = trace_lines(rows)
        with tempfile.TemporaryFile() as stream:
            out = Output(stream.fileno())
            out.put(control(START, 0, dict(record_bytes=64, rings=RINGS)))
            projector = Projector(out)
            for offset in range(0, len(raw), 777):
                projector.feed(raw[offset:offset+777])
            self.assertFalse(projector.pending)
            out.put(control(END, projector.total, dict(events=projector.total, clean=True,
                rings=RINGS, partial_bytes=0, event_counts=dict(projector.counts))), final=True)
            out.finish()
            stream.seek(0)
            decoded = [r for k, r in decode(stream) if k == DATA]
        self.assertEqual(rows, decoded)

    def test_caps_and_blocked_output(self):
        read_fd, write_fd = os.pipe()
        try:
            out = Output(write_fd, max_pending=4096)
            with self.assertRaises(CaptureError):
                for _ in range(10000):
                    out.put(b'x'*4096)
            self.assertLessEqual(out.pending, 4096)
            with self.assertRaises(CaptureError):
                out.finish(timeout=0.001)
        finally:
            os.close(read_fd)
            os.close(write_fd)
        with tempfile.TemporaryFile() as stream:
            with self.assertRaises(CaptureError):
                Output(stream.fileno(), max_bytes=65568).put(b'x')

    def test_socket_projection_no_addresses_or_unbounded_fields(self):
        text = 'ESTAB 0 0 192.168.77.2:5001 192.168.77.1:9876\n'
        text += ' cubic bytes_acked:123 notsent:42 unacked:2 cwnd:5 rtt:1.234/0.55 retrans:1/7 secret:888\n'
        row = project_sockets(text)[0]
        self.assertEqual(row['rtt_us'], 1234)
        self.assertEqual(row['retrans_total'], 7)
        self.assertNotIn('secret', row)
        self.assertEqual(project_sockets(''), [])

    def test_graceful_stop_drains_late_events_before_footer(self):
        closed = []
        class FakeTrace:
            owned = False
            fd = None
            writer = None
            def setup(self):
                self.fd, self.writer = os.pipe()
                os.set_blocking(self.fd, False)
                self.owned = True
            def enable(self):
                assert self.writer is not None
                os.write(self.writer, trace_lines(episode()))
            def disable(self):
                if self.writer is not None:
                    # A queued tail still exists when event production stops.
                    os.write(self.writer, trace_lines(episode(first=5, base=100)))
                    os.close(self.writer)
                    self.writer = None
            def close(self):
                self.disable()
                assert self.fd is not None
                os.close(self.fd)
                self.owned = False
                closed.append(True)
        class FakeWorker:
            pid = 1
            def __init__(self, **kwargs):
                self.keep = os.dup(kwargs['args'][0].fileno())
                self.alive = True
            def start(self):
                pass
            def is_alive(self):
                return self.alive
            def join(self, timeout=None):
                if self.alive:
                    os.close(self.keep)
                    self.alive = False
            def terminate(self):
                self.join()
        control_read, control_write = os.pipe()
        try:
            os.write(control_write, b'STOP\n')
            with tempfile.TemporaryFile() as output, mock.patch('collect_flow.Trace', FakeTrace), \
                    mock.patch('collect_flow.ring_stats', return_value=RINGS), \
                    mock.patch('collect_flow.multiprocessing.Process', FakeWorker), \
                    mock.patch('collect_flow.signal.signal'):
                self.assertEqual(collect(10, control_read, output.fileno(), use_native=False), 0)
                output.seek(0)
                result = analyze(output)
                self.assertEqual(result['counts']['matched_episodes'], 2)
                self.assertEqual(closed, [True])
        finally:
            os.close(control_read)
            os.close(control_write)


class ReceiverTests(unittest.TestCase):
    def test_private_complete_artifact_and_footer(self):
        raw = capture(episode())
        with tempfile.TemporaryDirectory() as directory:
            path = os.path.join(directory, 'capture.xtf')
            receiver = Receiver(io.BytesIO(raw), path)
            self.assertTrue(receiver.done.wait(timeout=2))
            self.assertIsNone(receiver.failure)
            self.assertTrue(receiver.ready.is_set())
            self.assertIsNotNone(receiver.end)
            self.assertEqual(receiver.byte_count, len(raw))
            self.assertEqual(os.stat(path).st_mode & 0o777, 0o600)
            with open(path, 'rb') as source:
                self.assertEqual(source.read(), raw)
            with self.assertRaises(CaptureError):
                Receiver(io.BytesIO(raw), path)

    def test_truncated_receiver_never_reports_complete(self):
        with tempfile.TemporaryDirectory() as directory:
            receiver = Receiver(io.BytesIO(capture(episode())[:-3]),
                                os.path.join(directory, 'truncated.xtf'))
            self.assertTrue(receiver.done.wait(timeout=2))
            self.assertIsNotNone(receiver.failure)
            self.assertIsNone(receiver.end)

    def test_disk_budget(self):
        tape = Tape(io.BytesIO(b'12345'), io.BytesIO(), limit=4)
        with self.assertRaises(CaptureError):
            tape.read(5)
        self.assertEqual(tape.bytes, 0)

    def test_progress_guard_requires_fresh_backlogged_samples(self):
        progress = Progress(5)
        def sample(second, acked=123, pending=5):
            return dict(after_ns=second*10**9, error=0,
                        sockets=[dict(bytes_acked=acked, notsent=pending)])
        for second in range(5):
            self.assertFalse(progress.stalled(sample(second)))
        self.assertTrue(progress.stalled(sample(5)))
        self.assertFalse(progress.stalled(sample(6, acked=124)))
        self.assertFalse(progress.stalled(sample(20, acked=124)))
        self.assertFalse(progress.stalled(sample(21, acked=124, pending=0)))
        self.assertFalse(progress.stalled(dict(after_ns=22*10**9, sockets=[], error=0)))

    def test_final_undrained_ring_is_rejected(self):
        rings = [dict(RINGS[0], entries=1)]
        raw = control(START, 0, dict(record_bytes=64, rings=RINGS))
        raw += control(END, 0, dict(events=0, clean=True, partial_bytes=0,
                                   event_counts={}, rings=rings))
        with self.assertRaises(CaptureError):
            list(decode(io.BytesIO(raw)))


@unittest.skipUnless(os.environ.get('XR819_FLOW_NATIVE_TEST_LIB'), 'run build-native.sh for native qualification')
class NativeTests(unittest.TestCase):
    def project_native(self, raw):
        with tempfile.TemporaryFile() as output:
            projector = NativeProjector(Output(output.fileno()), os.environ['XR819_FLOW_NATIVE_TEST_LIB'])
            for offset in range(0, len(raw), 4096):
                projector.feed(raw[offset:offset+4096])
            projector.flush_input()
            self.assertFalse(projector.pending)
            output.seek(0)
            from flow_format import HEADER
            events = []
            while header := output.read(HEADER.size):
                size = HEADER.unpack(header)[-2]
                payload = output.read(size)
                events.extend(Event.unpack(payload[i:i+64]) for i in range(0, size, 64))
            return events

    def test_differential_all_schemas_and_integer_boundaries(self):
        from flow_format import SCHEMAS, SIGNED
        generator = random.Random(819)
        rows = []
        for seq in range(1, 2001):
            kind = generator.choice(list(SCHEMAS))
            values = tuple(generator.randint(-2**31, 2**31-1) if key in SIGNED
                           else generator.randint(0, 2**32-1) for key in SCHEMAS[kind][1])
            rows.append(Event(seq, generator.randrange(2**64), generator.randrange(4), kind, values))
        self.assertEqual(self.project_native(trace_lines(rows)), rows)

    def test_staging_preserves_small_tail(self):
        with tempfile.TemporaryFile() as output:
            projector = NativeProjector(Output(output.fileno()), os.environ['XR819_FLOW_NATIVE_TEST_LIB'])
            projector.feed(trace_lines(episode()))
            self.assertEqual(projector.total, 0)
            projector.flush_input()
            self.assertEqual(projector.total, 4)
            self.assertEqual(projector.batches, 1)
            self.assertFalse(projector.input)

    def test_deployed_mmc_and_redaction(self):
        raw = (b'x [001] ... 7.500000: mmc_request_done: mmc1: end struct mmc_request[deadbeef]: '
               b'cmd_opcode=53 cmd_err=-110 cmd_resp=0xa 0xb 0xc 0xd stop_err=0 '
               b'bytes_xfered=0 data_err=-110 secret=1234\n')
        self.assertEqual(self.project_native(raw), [project(raw.decode().strip(), 1)])

    def test_rejects_bad_fields_and_keeps_valid_prefix(self):
        valid = trace_lines(episode())
        with tempfile.TemporaryFile() as output:
            projector = NativeProjector(Output(output.fileno()), os.environ['XR819_FLOW_NATIVE_TEST_LIB'])
            with self.assertRaises(CaptureError):
                projector.feed(valid + b'CPU:2 [LOST 130 EVENTS]\n')
                projector.flush_input()
            self.assertEqual(projector.total, 4)
            self.assertEqual(sum(projector.counts.values()), 4)
        for bad in (b'x [000] ... 1.0: net_dev_queue: len=1junk\n',
                    b'x [000] ... 1.0: net_dev_queue: len=1 len=2\n',
                    b'x [000] ... 1.0: net_dev_queue: len=4294967296\n'):
            with self.subTest(bad=bad), self.assertRaises(CaptureError):
                self.project_native(bad)


if __name__ == '__main__':
    unittest.main()
