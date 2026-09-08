#!/usr/bin/env python3
"""Versioned, bounded XR819 numeric metadata stream. No raw trace text on wire."""
import collections
import dataclasses
import json
import re
import struct
import zlib

SCHEMAS = {
    1: ('cw1200_queued', ('id', 'previous', 'len', 'queue', 'requeue')),
    2: ('cw1200_transport', ('id', 'msgid', 'len', 'data', 'done', 'result')),
    3: ('cw1200_confirmed', ('id', 'status', 'rate', 'ack', 'flags', 'media', 'queued')),
    4: ('net_dev_queue', ('len',)),
    5: ('mmc_request_done', ('cmd_opcode', 'cmd_err', 'data_err', 'stop_err', 'bytes_xfered')),
}
BY_NAME = {name: kind for kind, (name, _) in SCHEMAS.items()}
SIGNED = {'result', 'cmd_err', 'data_err', 'stop_err'}
RECORD = struct.Struct('<QQHH10II')  # 64 bytes; ten value slots and reserved zero
HEADER = struct.Struct('<4sBBHQQII')  # 32 bytes, including checksum
MAX_PAYLOAD = 65536
START, DATA, HEALTH, SOCKET, END = range(1, 6)
LINE = re.compile(r'\[(\d+)\].*?(\d+)\.(\d{1,9}):\s+(\w+):\s*(.*)$')
FIELD = re.compile(r'\b(\w+)=(-?(?:0x[0-9a-fA-F]+|[0-9]+))(?=\s|$)')


class CaptureError(ValueError):
    """Invalid or incomplete capture; never silently use it for timing results."""


@dataclasses.dataclass(frozen=True)
class Event:
    sequence: int
    ns: int
    cpu: int
    kind: int
    values: tuple

    def fields(self):
        return dict(zip(SCHEMAS[self.kind][1], self.values, strict=True))

    def pack(self):
        if self.kind not in SCHEMAS or len(self.values) != len(SCHEMAS[self.kind][1]):
            raise CaptureError('event schema')
        if not 0 < self.sequence < 2**64 or not 0 <= self.ns < 2**64 or not 0 <= self.cpu < 65536:
            raise CaptureError('event header range')
        for key, value in zip(SCHEMAS[self.kind][1], self.values, strict=True):
            lo, hi = (-2**31, 2**31) if key in SIGNED else (0, 2**32)
            if not lo <= value < hi:
                raise CaptureError('event field range')
        words = tuple(v & 0xffffffff for v in self.values)
        return RECORD.pack(self.sequence, self.ns, self.cpu, self.kind,
                           *words, *([0] * (10 - len(words))), 0)

    @classmethod
    def unpack(cls, raw):
        seq, ns, cpu, kind, *words = RECORD.unpack(raw)
        if kind not in SCHEMAS:
            raise CaptureError('unknown event type')
        keys = SCHEMAS[kind][1]
        if any(words[len(keys):]):
            raise CaptureError('nonzero reserved field')
        values = tuple(v - 2**32 if k in SIGNED and v >= 2**31 else v
                       for k, v in zip(keys, words[:len(keys)], strict=True))
        result = cls(seq, ns, cpu, kind, values)
        result.pack()  # Validate decoded ranges too.
        return result


def project(line, sequence):
    """Strict allowlist. Unknown/malformed/lost lines fail, never get forwarded."""
    if not line.strip() or line.startswith('#'):
        return None
    match = LINE.search(line)
    if not match or match[4] not in BY_NAME:
        raise CaptureError('unrecognized trace line or loss notice')
    kind = BY_NAME[match[4]]
    pairs = FIELD.findall(match[5])
    keys = SCHEMAS[kind][1]
    selected = {k: [] for k in keys}
    for key, value in pairs:
        if key in selected:
            selected[key].append(value)
    if any(len(v) != 1 for v in selected.values()):
        raise CaptureError('missing or duplicate permitted field')
    # Completion TP_printk omits cmd_arg; never derive direction from responses.
    try:
        values = tuple(int(selected[k][0], 16 if '0x' in selected[k][0] else 10) for k in keys)
        ns = int(match[2]) * 10**9 + int(match[3].ljust(9, '0'))
        event = Event(sequence, ns, int(match[1]), kind, values)
    except ValueError as exc:
        raise CaptureError('invalid numeric field') from exc
    event.pack()
    return event


def frame(kind, first, count, payload):
    if len(payload) > MAX_PAYLOAD or kind not in (START, DATA, HEALTH, SOCKET, END):
        raise CaptureError('frame limit/type')
    header = HEADER.pack(b'XTF1', 1, kind, 0, first, count, len(payload), 0)
    checksum = zlib.crc32(payload, zlib.crc32(header[:-4]))
    return header[:-4] + struct.pack('<I', checksum) + payload


def control(kind, total, fields):
    return frame(kind, total, 0, json.dumps(fields, sort_keys=True,
                                          separators=(',', ':'), allow_nan=False).encode('ascii'))


def data_frame(events):
    if not events:
        raise CaptureError('empty data batch')
    if any(e.sequence != events[0].sequence + i for i, e in enumerate(events)):
        raise CaptureError('noncontiguous batch')
    return frame(DATA, events[0].sequence, len(events), b''.join(e.pack() for e in events))


def read_exact(stream, length, eof_ok=False):
    chunks = bytearray()
    while len(chunks) < length:
        part = stream.read(length - len(chunks))
        if not part:
            if eof_ok and not chunks:
                return None
            raise CaptureError('truncated stream')
        chunks.extend(part)
    return bytes(chunks)


def validate_rings(obj):
    rings = obj.get('rings')
    if not isinstance(rings, list) or not rings or len(rings) > 256:
        raise CaptureError('missing or excessive ring statistics')
    cpus = set()
    for row in rings:
        if not isinstance(row, dict):
            raise CaptureError('ring schema')
        cpu = row.get('cpu')
        if type(cpu) is not int or not 0 <= cpu < 65536 or cpu in cpus:
            raise CaptureError('ring CPU identity')
        cpus.add(cpu)
        if type(row.get('entries')) is not int or row['entries'] < 0:
            raise CaptureError('missing ring occupancy counter')
        for key in ('overrun', 'commit_overrun', 'dropped_events'):
            if type(row.get(key)) is not int or row[key] != 0:
                raise CaptureError('missing ring counter or kernel loss')
    return cpus


def decode(stream):
    """Yield (kind, Event/control dict). Exhaust iterator to validate final EOF."""
    total = 0
    counts = collections.Counter()
    cpus = set()
    started = ended = False
    while True:
        raw = read_exact(stream, HEADER.size, eof_ok=True)
        if raw is None:
            if not ended:
                raise CaptureError('missing final footer')
            return
        magic, version, kind, reserved, first, count, size, checksum = HEADER.unpack(raw)
        if ended or magic != b'XTF1' or version != 1 or reserved or size > MAX_PAYLOAD:
            raise CaptureError('invalid frame header or trailing data')
        payload = read_exact(stream, size)
        assert payload is not None
        if zlib.crc32(payload, zlib.crc32(raw[:-4])) != checksum:
            raise CaptureError('frame checksum')
        if kind == DATA:
            if not started or not count or size != count * RECORD.size or first != total + 1:
                raise CaptureError('data length/sequence/start')
            for offset in range(0, size, RECORD.size):
                event = Event.unpack(payload[offset:offset + RECORD.size])
                total += 1
                if event.sequence != total or event.cpu not in cpus:
                    raise CaptureError('record sequence gap or unknown CPU')
                counts[str(event.kind)] += 1
                yield kind, event
        else:
            if count or first != total or kind not in (START, HEALTH, SOCKET, END):
                raise CaptureError('control header')
            try:
                obj = json.loads(payload)
            except (ValueError, UnicodeError) as exc:
                raise CaptureError('invalid control encoding') from exc
            if not isinstance(obj, dict):
                raise CaptureError('control schema')
            if kind == START:
                if started or total or obj.get('record_bytes') != RECORD.size:
                    raise CaptureError('start schema')
                cpus = validate_rings(obj)
                started = True
            elif not started:
                raise CaptureError('control before start')
            if kind in (HEALTH, END) and validate_rings(obj) != cpus:
                raise CaptureError('ring CPU set changed')
            if kind == SOCKET and obj.get('error') != 0:
                raise CaptureError('socket sampling failure')
            if kind == END:
                if obj.get('events') != total or type(obj.get('clean')) is not bool or not obj['clean']:
                    raise CaptureError('incomplete footer')
                if (obj.get('event_counts') != dict(counts) or obj.get('partial_bytes') != 0
                        or any(row['entries'] for row in obj['rings'])):
                    raise CaptureError('footer event accounting')
                ended = True
            yield kind, obj
