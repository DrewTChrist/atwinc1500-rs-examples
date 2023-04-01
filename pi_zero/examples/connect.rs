use std::error::Error;

use rppal::gpio::Gpio;
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

use atwinc1500::Atwinc1500;
use atwinc1500::wifi::Channel;
use atwinc1500::wifi::ConnectionParameters;

const GPIO_27: u8 = 27;
const GPIO_22: u8 = 22;
const GPIO_17: u8 = 17;
const GPIO_8: u8 = 8;
const GPIO_5: u8 = 5;

fn main() -> Result<(), Box<dyn Error>> {
    // Define pins
    let mut irq = Gpio::new()?.get(GPIO_27)?.into_input_pullup();
    let mut reset = Gpio::new()?.get(GPIO_22)?.into_output();
    let mut enable = Gpio::new()?.get(GPIO_17)?.into_output();
    let mut wake = Gpio::new()?.get(GPIO_5)?.into_output();
    let mut cs = Gpio::new()?.get(GPIO_8)?.into_output();

    // Define spi
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?;

    // Define delay
    let delay = Delay::new();

    // Turn off reset_on_drop
    irq.set_reset_on_drop(false);
    reset.set_reset_on_drop(false);
    enable.set_reset_on_drop(false);
    wake.set_reset_on_drop(false);
    cs.set_reset_on_drop(false);

    wake.toggle();


    let mut atwinc1500 = Atwinc1500::new(spi, delay, cs, irq, reset, enable, false).unwrap();

    // Read ssid from environment variable
    const SSID: &[u8] = "".as_bytes(); //core::env!("SSID").as_bytes();
    // Read password from environment variable
    const PASS: &[u8] = "".as_bytes(); //core::env!("PASS").as_bytes();

    // Connect to the network with our connection
    // parameters
    let connection = ConnectionParameters::wpa_psk(SSID, PASS, Channel::default(), 0);

    atwinc1500.connect_network(connection).unwrap();

    loop {}
}
