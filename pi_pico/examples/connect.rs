#![no_std]
#![no_main]

use cortex_m_rt::entry;
use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;
use fugit::ExtU32;
use fugit::RateExtU32;
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::gpio::Interrupt::EdgeLow;

use bsp::hal::pac::interrupt;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio::{self, bank0::Gpio22, Input, dynpin::DynPin, Pin, PullUp},
    pac,
    sio::Sio,
    spi,
    timer::Timer,
    watchdog::Watchdog,
};

use atwinc1500::wifi::Channel;
use atwinc1500::wifi::Connection;
use atwinc1500::Atwinc1500;
use atwinc1500::Status;

use core::cell::RefCell;
use core::env;

type Atwinc = Atwinc1500<spi::Spi<spi::Enabled, pac::SPI0, 8>, cortex_m::delay::Delay, DynPin>;
type IrqPin = Pin<Gpio22, Input<PullUp>>;

// Define static variables to be passed to the interrupt
static ATWINC: Mutex<RefCell<Option<Atwinc>>> = Mutex::new(RefCell::new(None));
static IRQ_PIN: Mutex<RefCell<Option<IrqPin>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    info!("Program start");
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut count_down = timer.count_down();

    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let _spi_sclk = pins.gpio18.into_mode::<gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio19.into_mode::<gpio::FunctionSpi>();
    let _spi_miso = pins.gpio16.into_mode::<gpio::FunctionSpi>();
    let spi = spi::Spi::<_, _, 8>::new(pac.SPI0);
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        11_000_000u32.Hz(),
        &embedded_hal::spi::MODE_0,
    );

    let mut onboard_led = pins.led.into_push_pull_output();
    onboard_led.set_high().unwrap();

    let cs: gpio::DynPin = pins.gpio17.into_push_pull_output().into();
    let irq: IrqPin = pins.gpio22.into_pull_up_input().into();
    irq.set_interrupt_enabled(EdgeLow, true);
    let reset: gpio::DynPin = pins.gpio21.into_push_pull_output().into();
    let mut en_wake: gpio::DynPin = pins.gpio20.into_push_pull_output().into();
    en_wake.set_high().unwrap();

    let mut atwinc1500 = Atwinc1500::new(spi, delay, cs, reset, false);
    atwinc1500.initialize().unwrap();

    info!("Create atwinc struct");

    // Read ssid from environment variable
    const SSID: &[u8] = env!("SSID").as_bytes();
    // Read password from environment variable
    const PASS: &[u8] = env!("PASS").as_bytes();

    // Create connection parameters
    let connection = Connection::wpa_psk(SSID, PASS, Channel::default(), 0);

    critical_section::with(|cs| {
        // Store the driver and irq pin in
        // the static variables
        ATWINC.borrow(cs).replace(Some(atwinc1500));
        IRQ_PIN.borrow(cs).replace(Some(irq));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    critical_section::with(|cs| {
        if let Some(atwinc) = ATWINC.borrow(cs).take() {
            // Check status of atwinc
            // it should be idle
            info!("Status: {:?}", atwinc.get_status());
            ATWINC.borrow(cs).replace(Some(atwinc));
        }
    });

    critical_section::with(|cs| {
        if let Some(mut atwinc) = ATWINC.borrow(cs).take() {
            // Connect to the network with our connection parameters
            atwinc.connect_network(connection).unwrap();
            ATWINC.borrow(cs).replace(Some(atwinc));
        }
    });

    count_down.start(6000u32.millis());
    let _ = nb::block!(count_down.wait());

    critical_section::with(|cs| {
        if let Some(atwinc) = ATWINC.borrow(cs).take() {
            info!("{:?}", atwinc.get_status());
        }
    });

    loop {}
}

#[interrupt]
fn IO_IRQ_BANK0() {
    info!("Enter Interrupt");
    critical_section::with(|cs| {
        // Take the driver an interrupt request pin 
        let winc = ATWINC.borrow(cs).take();
        let irq = IRQ_PIN.borrow(cs).take();

        if let Some(mut atwinc) = winc {
            // Handle driver events and then
            // put it back in the static variable
            atwinc.handle_events().unwrap();
            ATWINC.borrow(cs).replace_with(|_| Some(atwinc));
        }

        if let Some(mut irq_pin) = irq {
            // Clear our interrupt and 
            // put the pin back in the static variable
            irq_pin.clear_interrupt(EdgeLow);
            IRQ_PIN.borrow(cs).replace_with(|_| Some(irq_pin));
        }
    });
    info!("Exit interrupt");
}
