#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use defmt_rtt as _;
use embedded_time::fixed_point::FixedPoint;
use embedded_time::rate::Extensions;
use panic_probe as _;

use rp_pico as bsp;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    gpio, pac,
    sio::Sio,
    spi,
    watchdog::Watchdog,
};

use atwinc1500::Atwinc1500;

#[entry]
fn main() -> ! {
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

    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());

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

    let cs: gpio::DynPin = pins.gpio17.into_push_pull_output().into();
    let irq: gpio::DynPin = pins.gpio22.into_pull_up_input().into();
    let reset: gpio::DynPin = pins.gpio21.into_push_pull_output().into();
    let en_wake: gpio::DynPin = pins.gpio20.into_push_pull_output().into();

    let mut atwinc1500 = Atwinc1500::new(spi, delay, cs, irq, reset, en_wake, false).unwrap();

    // Get and print the version of the firmware
    // running on the Atwinc1500
    let version = atwinc1500.get_firmware_version().unwrap();
    info!("Firmware Version: {}", version);

    // Get and print the mac address
    // of the Atwinc1500
    let mac = atwinc1500.get_mac_address().unwrap();
    info!("Mac Address: {}", mac);

    loop {}
}
