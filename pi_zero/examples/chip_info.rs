use std::error::Error;

use rppal::gpio::Gpio;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::hal::Delay;

use atwinc1500::Atwinc1500;

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

    let atwinc1500 = Atwinc1500::new(spi, delay, cs, irq, reset, enable, false);

    match atwinc1500 {
        Ok(mut at) => {
            // Get and print the version of the firmware
            // running on the Atwinc1500
            if let Ok(fw) = at.get_firmware_version() {
                println!("Firmware Version: {}", fw);
            }
             
            // Get and print the mac address
            // of the Atwinc1500
            if let Ok(mac) = at.get_mac_address() {
                println!("Mac Address: {}", mac);
            }
        }
        Err(e) => println!("{}", e),
    }

    loop {}
}
