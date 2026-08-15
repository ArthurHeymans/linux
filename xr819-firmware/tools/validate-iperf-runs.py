import re,sys,glob
print(f"{'run':16} {'TCP':>10} {'UDP delivered':>14} {'loss':>8} {'TXed(iperf)':>12}  verdict")
for f in sorted(glob.glob('/tmp/rs[0-9].txt')+glob.glob('/tmp/ce[0-9].txt')+glob.glob('/tmp/rx[0-9].txt')+glob.glob('/tmp/u[0-9].txt')):
    t=open(f, errors='replace').read()
    if '=== UDP' not in t: continue
    tcp_fail = 'connect to' in t and 'failed' in t
    m=re.search(r'([\d.]+) (?:Mbits|Kbits)/sec\s+[\d.]+ ms\s+(\d+)/\s*(\d+)', t)   # server report has jitter+loss
    txs=[int(x) for x in re.findall(r'^TXed:\s+(\d+)', t, re.M)]
    dtx = (max(txs)-min(txs)) if txs else 0
    tcpv=re.findall(r'0\.0000-\d+\.\d+ sec\s+[\d.]+ [MK]Bytes\s+([\d.]+) (Mbits|Kbits)', t)
    tcp = f"{tcpv[0][0]}{'M' if tcpv[0][1]=='Mbits' else 'K'}" if tcpv and not tcp_fail else "FAILED"
    if m:
        rate, lost, tot = m.group(1), int(m.group(2)), int(m.group(3))
        verdict = "VALID" if dtx > 500 else "suspect (no TX)"
        print(f"{f.split('/')[-1][:-4]:16} {tcp:>10} {rate+' Mbit':>14} {100*lost/tot:7.1f}% {dtx:>12}  {verdict}")
    else:
        print(f"{f.split('/')[-1][:-4]:16} {tcp:>10} {'no server report':>14} {'-':>8} {dtx:>12}  INVALID: link dead, UDP wrote into the void")
