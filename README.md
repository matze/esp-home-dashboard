# ESP Home Dashboard

An embedded Rust project for ESP32-C3 that displays calendar events and weather
on a 7.5" e-paper display. It fetches calendar data via WiFi from an iCal URL
and synchronizes time via NTP.

## Hardware

- ESP32-C3 (RISC-V)
- Waveshare 7.5" e-paper display (V2)

### Pin Mapping

| ESP32-C3 | Display | Function    |
|----------|---------|-------------|
| GPIO0    | DC      | Data/Command|
| GPIO1    | BUSY    | Busy signal |
| GPIO3    | RST     | Reset       |
| GPIO6    | CLK     | SPI clock   |
| GPIO7    | DIN     | SPI MOSI    |
| GPIO10   | CS      | Chip select |

Connect display VCC to 3.3V and GND to ground.

## Flashing

Environment variables are embedded at compile time:

```bash
WIFI_SSID="..." WIFI_PASSWORD="..." ICAL_URL="..." WEATHER_LAT="..." WEATHER_LON="..." cargo run --release
```

## License

MIT
