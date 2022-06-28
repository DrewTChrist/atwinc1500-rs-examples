use std::error::Error;

use rppal::gpio::Gpio;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::hal::Delay;

use atwinc1500::Atwinc1500;
use atwinc1500::gpio::AtwincGpio;
use atwinc1500::gpio::GpioDirection;
use atwinc1500::gpio::GpioValue;

const GPIO_27: u8 = 27;
const GPIO_22: u8 = 22;
const GPIO_17: u8 = 17;
const GPIO_8: u8 = 8;
const GPIO_5: u8 = 5;

fn main() -> Result<(), Box<dyn Error>> {
    let irq = Gpio::new()?.get(GPIO_27)?.into_input_pullup();
    let reset = Gpio::new()?.get(GPIO_22)?.into_output();
    let enable = Gpio::new()?.get(GPIO_17)?.into_output();
    let mut wake = Gpio::new()?.get(GPIO_5)?.into_output();
    let cs = Gpio::new()?.get(GPIO_8)?.into_output();
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 8_000_000, Mode::Mode0)?;
    let delay = Delay::new();
    wake.toggle();
    let atwinc1500 = Atwinc1500::new(spi, delay, cs, irq, reset, enable, false);
    match atwinc1500 {
        Ok(mut at) => {
            at.set_gpio_direction(AtwincGpio::Gpio4, GpioDirection::Output);
            at.set_gpio_value(AtwincGpio::Gpio4, GpioValue::High);
        },
        Err(e) => panic!("{}", e),
    }
    loop {}
    //Ok(())
}
