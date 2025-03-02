#!/bin/bash
modprobe spidev
modprobe spi_bcm2835
modprobe spi_dw_mmio
modprobe spi_dw

while ! ( find /dev -maxdepth 1 -name 'spidev*' | grep -F '' > /dev/null ) ; do
    echo "Waiting spidevs"
    sleep 1
done

while ! [ -e /dev/gpiochip4 ]; do
	echo "Waiting gpiochip"
	sleep 1
done

export RUST_BACKTRACE=1
cd /opt/astro-epd-display/ || exit 1
exec /opt/astro-epd-display/astro-epd-display --json '{"rotate": 1}' --template /opt/astro-epd-display/template.yaml --scrape-command /opt/astro-epd-display/scrape.py epd
