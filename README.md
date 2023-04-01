# atwinc1500-rs-examples

## Description
This is a set of examples for the atwinc1500 crate using different host targets. 

## Example Configurations
### Raspberry Pi Pico + Adafruit Atwinc1500 Breakout Board
#### Wiring
|Host Pin|Breakout pin|
|---|---|
|3V3(OUT)|Vin|
|ground|GND|
|Gpio 18|SCK|
|Gpio 16|MISO|
|Gpio 19|MOSI|
|Gpio 17|CS|
|Gpio 20|EN|
|Gpio 22|IRQ|
|Gpio 21|RST|
|Gpio 20|WAKE|

#### Examples
* chip_info - Get the firmware version and mac address of the Atwinc1500
* connect - Connect to a network with wpa2
* led - Control an led on the Adafruit Atwinc1500 Breakout
* scan - Scan for networks
* time - Get the system time from the Atwinc1500 sntp client

### Raspberry Pi Zero 1.3 + Adafruit Atwinc1500 Breakout Board
#### Wiring
|Host Pin|Breakout pin|
|---|---|
|3.3v|Vin|
|ground|GND|
|Gpio 11|SCK|
|Gpio 9|MISO|
|Gpio 10|MOSI|
|Gpoi 8|CS|
|Gpio 17|EN|
|Gpio 27|IRQ|
|Gpio 22|RST|
|Gpio 5|WAKE|

#### Examples
* chip_info - Get the firmware version and mac address of the Atwinc1500
* connect - Connect to a network with wpa2
* led - Control an led on the Adafruit Atwinc1500 Breakout

### STM32F3 Discovery Board + Adafruit Atwinc1500 Breakout Board
#### Wiring
|Host Pin|Breakout pin|
|---|---|
|3V|Vin|
|ground|GND|
|pc10|SCK|
|pc11|MISO|
|pc12|MOSI|
|pe8|CS|
|pe10|EN|
|pe11|IRQ|
|pe9|RST|
|pe10|WAKE|

#### Examples
* chip_info - Get the firmware version and mac address of the Atwinc1500
* connect - Connect to a network with wpa2
* led - Control an led on the Adafruit Atwinc1500 Breakout
