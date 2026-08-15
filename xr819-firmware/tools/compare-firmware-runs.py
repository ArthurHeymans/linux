#!/usr/bin/env python3
"""Compare firmware builds on validated iperf runs plus driver-visible faults.

A UDP run only counts when iperf printed a server report (jitter + lost/total)
AND the firmware's TXed counter moved across the phase; without both, a dead
link reports the offered rate and looks like a record.

Firmware exceptions are counted from the driver's bh_rx_diag ring, which
records WSM id 0x0800 (reason=5) whatever firmware produced it, so vendor and
our builds are directly comparable.
"""
import re, sys

def runs(text, marker):
    parts = re.split(r'#{5,} (.+?) #{5,}', text)
    return [(parts[i].strip(), parts[i + 1]) for i in range(1, len(parts) - 1, 2)]

def summarise(name, body):
    server = re.search(r'([\d.]+) (?:Mbits|Kbits)/sec\s+[\d.]+ ms\s+(\d+)/\s*(\d+)', body)
    txs = [int(x) for x in re.findall(r'^TXed:\s+(\d+)', body, re.M)]
    dtx = max(txs) - min(txs) if txs else 0
    tcp = re.search(r'0\.0000-\d+\.\d+ sec\s+[\d.]+ [MK]Bytes\s+([\d.]+) (Mbits|Kbits)', body)
    tcp_failed = 'failed' in body and 'connect to' in body
    exceptions = len(re.findall(r'reason=5', body))
    diag = len(re.findall(r'BH RX diag', body))
    fatal = len(re.findall(r'Fatal error', body))
    ping = re.search(r'([\d.]+)% packet loss', body)
    valid = bool(server) and dtx > 500
    udp = f"{server.group(1)} Mbit" if server else "NONE"
    loss = f"{100*int(server.group(2))/int(server.group(3)):.0f}%" if server else "-"
    tcpv = "FAILED" if tcp_failed else (f"{tcp.group(1)}{tcp.group(2)[0]}" if tcp else "-")
    print(f"{name:22} {tcpv:>8} {udp:>11} {loss:>6} {dtx:>8} {exceptions:>6} {diag:>5} {fatal:>6} "
          f"{ping.group(1)+'%' if ping else '-':>7}  {'VALID' if valid else 'invalid (dead link)'}")

for path in sys.argv[1:]:
    text = open(path, errors='replace').read()
    print(f"\n=== {path} ===")
    print(f"{'run':22} {'TCP':>8} {'UDP recvd':>11} {'loss':>6} {'TXed':>8} {'exc':>6} {'diag':>5} {'fatal':>6} {'ping':>7}")
    for name, body in runs(text, None):
        summarise(name, body)
