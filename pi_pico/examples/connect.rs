#![no_std]
#![no_main]

use cortex_m_rt::entry;
use critical_section::Mutex;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::v2::OutputPin;
use fugit::RateExtU32;
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::gpio::Interrupt::EdgeLow;

use bsp::hal::pac::interrupt;
use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio,
    gpio::dynpin::DynPin,
    pac,
    sio::Sio,
    spi,
    watchdog::Watchdog,
};

use atwinc1500::wifi::Channel;
use atwinc1500::wifi::Connection;
use atwinc1500::Atwinc1500;

use core::cell::RefCell;
use core::env;

type Atwinc =
    Atwinc1500<'static, spi::Spi<spi::Enabled, pac::SPI0, 8>, cortex_m::delay::Delay, DynPin>;
type IrqPin = bsp::hal::gpio::Pin<
    bsp::hal::gpio::bank0::Gpio22,
    bsp::hal::gpio::Input<bsp::hal::gpio::PullUp>,
>;
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
    let irq: bsp::hal::gpio::Pin<
        bsp::hal::gpio::bank0::Gpio22,
        bsp::hal::gpio::Input<bsp::hal::gpio::PullUp>,
    > = pins.gpio22.into_pull_up_input().into();
    irq.set_interrupt_enabled(EdgeLow, true);
    let reset: gpio::DynPin = pins.gpio21.into_push_pull_output().into();
    let mut en_wake: gpio::DynPin = pins.gpio20.into_push_pull_output().into();
    en_wake.set_high().unwrap();

    let atwinc1500 = Atwinc1500::new(spi, delay, cs, reset, false);

    info!("Create atwinc struct");

    // Read ssid from environment variable
    const SSID: &[u8] = env!("SSID").as_bytes();
    // Read password from environment variable
    const PASS: &[u8] = env!("PASS").as_bytes();

    // Connect to the network with our connection
    // parameters
    let connection = Connection::wpa_psk(SSID, PASS, Channel::default(), 0);

    critical_section::with(|cs| {
        ATWINC.borrow(cs).replace(Some(atwinc1500));
        IRQ_PIN.borrow(cs).replace(Some(irq));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    critical_section::with(|cs| {
        if let Some(mut atwinc) = ATWINC.borrow(cs).take() {
            atwinc.initialize().unwrap();
        }
    });

    critical_section::with(|cs| {
        if let Some(mut atwinc) = ATWINC.borrow(cs).take() {
            atwinc.connect_network(connection).unwrap();
        }
    });

    critical_section::with(|cs| {
        if let Some(mut atwinc) = ATWINC.borrow(cs).take() {
            info!("{:?}", atwinc.get_status());
        }
    });

    loop {}
}

#[interrupt]
fn IO_IRQ_BANK0() {
    static mut WINC: Option<Atwinc> = None;
    static mut IRQ: Option<IrqPin> = None;

    if WINC.is_none() {
        critical_section::with(|cs| {
            *WINC = ATWINC.borrow(cs).take();
        });
    }
    if IRQ.is_none() {
        critical_section::with(|cs| {
            *IRQ = IRQ_PIN.borrow(cs).take();
        });
    }
    info!("Interrupt");

    if let Some(atwinc) = WINC {
        atwinc.handle_events().unwrap();
    }

    if let Some(irq) = IRQ {
        irq.clear_interrupt(EdgeLow);
    }
}
