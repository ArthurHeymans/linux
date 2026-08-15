# Recovered hardware data

`aes-mode1.bin` is the 430-byte AES-engine program loaded by the vendor routine
whose Thumb body appears at container file offset `0x179fc`. It was extracted
from DTCM `0x04000830..0x040009de` in the local vendor firmware container:

- firmware SHA-256: `fb81436ad7cc0876614a2a9c2a54c5a93a75315aee164e3a3afe3db80842a9e1`
- container data offset: `0x01cdf8 + 0x830`
- microcode SHA-256: `211ad6ec637a780d770998fbe76bcefa1b127506ef8e0f9f09539da32d881b4c`

The loader writes `0x80000000 | aes-mode1.bin[0]` to `AES+0x08`, streams the
remaining bytes to the same port, then issues command `0x9000`. No XR819 details
were sourced from the web.
