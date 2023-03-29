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
    gpio::{
        self,
        bank0::{Gpio22, Gpio25},
        dynpin::DynPin,
        Input, Output, Pin, PullUp, PushPull,
    },
    pac,
    sio::Sio,
    spi,
    timer::Timer,
    watchdog::Watchdog,
};

use atwinc1500::wifi::Channel;
use atwinc1500::Atwinc1500;

use core::cell::RefCell;

type Atwinc = Atwinc1500<spi::Spi<spi::Enabled, pac::SPI0, 8>, cortex_m::delay::Delay, DynPin>;
type IrqPin = Pin<Gpio22, Input<PullUp>>;
type Led = Pin<Gpio25, Output<PushPull>>;

// Define static variables to be passed to the interrupt
static ATWINC: Mutex<RefCell<Option<Atwinc>>> = Mutex::new(RefCell::new(None));
static IRQ_PIN: Mutex<RefCell<Option<IrqPin>>> = Mutex::new(RefCell::new(None));
static LED: Mutex<RefCell<Option<Led>>> = Mutex::new(RefCell::new(None));

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

    let onboard_led = pins.led.into_push_pull_output();

    let cs: gpio::DynPin = pins.gpio17.into_push_pull_output().into();
    let irq: IrqPin = pins.gpio22.into_pull_up_input().into();
    irq.set_interrupt_enabled(EdgeLow, true);
    let reset: gpio::DynPin = pins.gpio21.into_push_pull_output().into();
    let mut en_wake: gpio::DynPin = pins.gpio20.into_push_pull_output().into();
    en_wake.set_high().unwrap();

    let mut atwinc1500 = Atwinc1500::new(spi, delay, cs, reset, false);
    //atwinc1500.initialize().unwrap();
    match atwinc1500.initialize() {
        Ok(()) => {}
        Err(e) => info!("{}", e),
    }

    critical_section::with(|cs| {
        // Store the driver and irq pin in
        // the static variables
        ATWINC.borrow(cs).replace(Some(atwinc1500));
        IRQ_PIN.borrow(cs).replace(Some(irq));
        LED.borrow(cs).replace(Some(onboard_led));
    });

    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    critical_section::with(|cs| {
        if let Some(mut atwinc) = ATWINC.borrow(cs).take() {
            // request network scan
            match atwinc.request_network_scan(Channel::default()) {
                Ok(_) => {}
                Err(e) => info!("Error requesting network scan: {}", e),
            }
            ATWINC.borrow(cs).replace(Some(atwinc));
        }
    });

    count_down.start(2000u32.millis());
    let _ = nb::block!(count_down.wait());

    let mut net_counter: u8 = 0;
    let mut leave = false;

    loop {
        critical_section::with(|cs| {
            if let Some(mut atwinc) = ATWINC.borrow(cs).take() {
                if atwinc.num_ap() > 0 {
                    if net_counter < atwinc.num_ap() {
                        atwinc.request_scan_result(net_counter).unwrap();
                        net_counter += 1;
                    } else {
                        leave = true;
                    }
                }
                ATWINC.borrow(cs).replace(Some(atwinc));
            }
        });
        if leave {
            break;
        }
        count_down.start(500u32.millis());
        let _ = nb::block!(count_down.wait());
        critical_section::with(|cs| {
            if let Some(atwinc) = ATWINC.borrow(cs).take() {
                if !atwinc.scan_result().is_none() {
                    info!("{:?}", atwinc.scan_result());
                } else {
                    info!("No scan result");
                }
                ATWINC.borrow(cs).replace(Some(atwinc));
            }
        });
    }

    loop {}
}

#[interrupt]
fn IO_IRQ_BANK0() {
    info!("Enter Interrupt");
    critical_section::with(|cs| {
        // Take the driver an interrupt request pin
        let irq = IRQ_PIN.borrow(cs).take();
        let (winc, led) = (ATWINC.borrow(cs).take(), LED.borrow(cs).take());

        if let (Some(mut atwinc), Some(mut onboard_led)) = (winc, led) {
            onboard_led.set_high().unwrap();
            // Handle driver events and then
            // put it back in the static variable
            match atwinc.handle_events() {
                Ok(_) => {}
                Err(e) => info!("Error handling events in interrupt: {}", e),
            }
            onboard_led.set_low().unwrap();
            ATWINC.borrow(cs).replace_with(|_| Some(atwinc));
            LED.borrow(cs).replace_with(|_| Some(onboard_led));
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
