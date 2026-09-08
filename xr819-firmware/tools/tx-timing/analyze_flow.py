#!/usr/bin/env python3
"""Streaming episode analysis. Uses host timestamps, never firmware delay units."""
import argparse
import collections
import json
from typing import Any

from flow_format import CaptureError, DATA, Event, decode


class Distribution:
    """Bounded eight-subdivisions-per-octave histogram, with honest quantile bounds."""
    def __init__(self):
        self.count = self.total = 0
        self.minimum = self.maximum = None
        self.bins = collections.Counter()

    def add(self, ns):
        if ns < 0 or ns >= 2**64:
            raise CaptureError('duration range')
        shift = max(0, ns.bit_length() - 4)
        lo = (ns >> shift) << shift
        hi = lo + (1 << shift) - 1
        self.bins[lo, hi] += 1
        self.count += 1
        self.total += ns
        self.minimum = ns if self.minimum is None else min(self.minimum, ns)
        self.maximum = ns if self.maximum is None else max(self.maximum, ns)

    def report(self):
        result = dict(count=self.count, sum_ns=self.total,
                      min_ns=self.minimum, max_ns=self.maximum)
        for percentile in (50, 90, 99):
            target = (self.count * percentile + 99) // 100
            seen = 0
            for bounds, count in sorted(self.bins.items()):
                seen += count
                if seen >= target:
                    result[f'p{percentile}_bounds_ns'] = bounds
                    break
        return result


class Matcher:
    def __init__(self, max_live=4096, max_strata=2048):
        self.max_live, self.max_strata = max_live, max_strata
        self.live = {}
        self.requeues = {}
        self.counts = collections.Counter()
        self.metrics = collections.defaultdict(Distribution)
        self.strata = {}
        self.cpu_last = {}

    def episode(self, cookie):
        if cookie not in self.live:
            if len(self.live) >= self.max_live:
                raise CaptureError('live episode cap; possible missing terminal events')
            self.live[cookie] = {}
        return self.live[cookie]

    def accept(self, event: Event):
        if event.ns < self.cpu_last.get(event.cpu, 0):
            raise CaptureError('per-CPU timestamp regression')
        self.cpu_last[event.cpu] = event.ns
        self.counts['events'] += 1
        row = event.fields()
        if event.kind not in (1, 2, 3):
            return
        if event.kind == 2 and not row['data']:
            self.counts['command_transport_events'] += 1
            return
        cookie = row['id']
        ep = self.episode(cookie)
        if event.kind == 1:
            stage = 'admit'
            if row['requeue']:
                previous = self.requeues.pop(row['previous'], None)
                self.counts['linked_requeues' if previous is not None else 'orphan_requeues'] += 1
        elif event.kind == 2:
            stage = 'end' if row['done'] else 'start'
            # A failed call may be retried without another queue admission.
            if stage == 'start' and 'end' in ep and ep['end'].fields()['result'] != 0:
                if 'start' in ep:
                    delta = ep['end'].ns - ep['start'].ns
                    if delta < 0:
                        raise CaptureError('failed write timestamp order')
                    self.metrics['failed_write_call'].add(delta)
                self.counts['failed_write_attempts'] += 1
                ep.pop('start', None)
                ep.pop('end', None)
                ep['retried'] = True
        else:
            stage = 'confirm'
        if stage in ep:
            raise CaptureError('ambiguous active cookie or duplicate stage')
        ep[stage] = event
        if all(key in ep for key in ('admit', 'start', 'end', 'confirm')):
            self.finish(cookie, ep)

    def finish(self, cookie, ep):
        admit, start, end, confirm = (ep[k] for k in ('admit', 'start', 'end', 'confirm'))
        a, s, e, c = (item.fields() for item in (admit, start, end, confirm))
        if s['msgid'] != e['msgid'] or s['len'] != e['len']:
            raise CaptureError('transport pair mismatch')
        del self.live[cookie]
        if start.ns < admit.ns or end.ns < start.ns or confirm.ns < start.ns:
            self.counts['censored_timestamp_order'] += 1
            return
        if e['result']:
            self.counts['censored_confirmation_after_failed_write'] += 1
            return
        durations = dict(write_call=end.ns-start.ns,
                         start_to_confirmation=confirm.ns-start.ns,
                         admission_to_confirmation=confirm.ns-admit.ns)
        if not ep.get('retried'):
            durations['admission_to_write_start'] = start.ns-admit.ns
        response = confirm.ns-end.ns
        durations['response_overlap' if response < 0 else 'write_return_to_confirmation'] = abs(response)
        key = (a['queue'], a['len'].bit_length(), c['rate'], c['status'], c['flags'])
        if key not in self.strata:
            if len(self.strata) >= self.max_strata:
                raise CaptureError('stratum cap')
            self.strata[key] = collections.defaultdict(Distribution)
        for name, duration in durations.items():
            self.metrics[name].add(duration)
            self.strata[key][name].add(duration)
        self.counts['matched_episodes'] += 1
        if c['status'] == 11 and c['flags'] & 2:
            if cookie in self.requeues or len(self.requeues) >= self.max_live:
                raise CaptureError('ambiguous or excessive pending requeues')
            self.requeues[cookie] = admit.sequence

    def report(self) -> dict[str, Any]:
        return dict(
            counts=dict(self.counts), censored_live_episodes=len(self.live),
            unlinked_requeue_confirmations=len(self.requeues),
            metrics={k: v.report() for k, v in self.metrics.items()},
            strata=[dict(queue=k[0], length_bit_length=k[1], rate=k[2], status=k[3],
                         flags=k[4], metrics={name: dist.report() for name, dist in values.items()})
                    for k, values in sorted(self.strata.items())],
            limitations=['Quantiles are histogram bounds, not exact percentiles.',
                         'Confirmation is not packet delivery or terminal queue removal.',
                         'No complete ownership/occupancy or TCP-to-cookie identity is inferred.'])


def analyze(stream) -> dict[str, Any]:
    matcher = Matcher()
    controls = collections.Counter()
    for kind, item in decode(stream):
        if kind == DATA:
            assert isinstance(item, Event)
            matcher.accept(item)
        else:
            controls[kind] += 1
    result = matcher.report()
    result['control_records'] = dict(controls)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('capture')
    args = parser.parse_args()
    try:
        with open(args.capture, 'rb') as stream:
            result = analyze(stream)
    except (CaptureError, OSError) as exc:
        parser.exit(1, f'Capture rejected: {exc}\n')
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
