use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

use rppal::gpio::{Gpio, Trigger};
use rppal::hal::Delay;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

use atwinc1500::wifi::Channel;
use atwinc1500::wifi::Connection;
use atwinc1500::Atwinc1500;

const GPIO_27: u8 = 27;
const GPIO_22: u8 = 22;
const GPIO_17: u8 = 17;
const GPIO_8: u8 = 8;
const GPIO_5: u8 = 5;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Setting up pins");
    // Define pins
    let mut irq = Gpio::new()?.get(GPIO_27)?.into_input_pullup();
    let mut reset = Gpio::new()?.get(GPIO_22)?.into_output();
    let mut enable = Gpio::new()?.get(GPIO_17)?.into_output();
    let mut wake = Gpio::new()?.get(GPIO_5)?.into_output();
    let mut cs = Gpio::new()?.get(GPIO_8)?.into_output();

    // Define spi
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;

    // Define delay
    let delay = Delay::new();

    // Turn off reset_on_drop
    irq.set_reset_on_drop(false);
    reset.set_reset_on_drop(false);
    enable.set_reset_on_drop(false);
    wake.set_reset_on_drop(false);
    cs.set_reset_on_drop(false);

    enable.toggle();
    wake.toggle();

    let mut atwinc1500 = Atwinc1500::new(spi, delay, cs, reset, false);
    atwinc1500.initialize().unwrap();
    let atwinc = Arc::new(Mutex::new(atwinc1500));

    let atwinc_clone = Arc::clone(&atwinc);
    irq.set_async_interrupt(Trigger::FallingEdge, move |_| {
        let mut winc = atwinc_clone.lock().unwrap();
        winc.handle_events().unwrap();
        println!("Handling events");
    })
    .unwrap();

    // Read ssid from environment variable
    const SSID: &[u8] = "".as_bytes(); //core::env!("SSID").as_bytes();
                                               // Read password from environment variable
    const PASS: &[u8] = "".as_bytes(); //core::env!("PASS").as_bytes();

    // Connect to the network with our connection
    // parameters
    let connection = Connection::wpa_psk(SSID, PASS, Channel::default(), 0);

    {
        let mut winc = atwinc.lock().unwrap();
        println!("Connecting to network");
        winc.connect_network(connection).unwrap();
    }

    println!("Sleeping...");
    thread::sleep(time::Duration::from_millis(5000));

    {
        let mut winc = atwinc.lock().unwrap();
        println!("Requesting connection info");
        winc.request_connection_info().unwrap();
    }

    println!("Sleeping...");
    thread::sleep(time::Duration::from_millis(5000));

    let tries = 200;
    let mut i = 0;

    loop {
        let winc = atwinc.lock().unwrap();
        if let Some(info) = winc.get_connection_info() {
            println!("{:?}", info);
            break;
        }
        if i < tries {
            i += 1;
        } else {
            break;
        }
    }

    Ok(())
}
