#!/bin/bash

set -euo pipefail

mkdir --parents --mode=755 /opt/astro-epd-display
install -o 0 -g 0 --mode=755 target/release/astro-epd-display /opt/astro-epd-display/
install -o 0 -g 0 --mode=755 systemd/start-epd.sh /opt/astro-epd-display/
install -o 0 -g 0 --mode=644 mobindi/template.yaml /opt/astro-epd-display/
install -o 0 -g 0 --mode=755 mobindi/scrape.py /opt/astro-epd-display/
install -o 0 -g 0 --mode=644 systemd/astro-epd-display.service /etc/systemd/system/

mkdir --parents --mode=755 /opt/astro-epd-display/resources
for I in resources/*; do
    install -o 0 -g 0 --mode=644 $I /opt/astro-epd-display/resources/
done
# This will fail if systemd is not running (e.g. in a chroot)
systemctl daemon-reload || true
systemctl enable astro-epd-display.service

