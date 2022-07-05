#![no_std]
#![no_main]

use panic_halt as _;
use defmt::*;
use defmt_rtt as _;
use stm32f3xx_hal as hal;

use cortex_m::asm;
use cortex_m_rt::entry;

use hal::pac;
use hal::prelude::*;
use hal::spi::Spi;
use hal::delay::Delay;

use atwinc1500::Atwinc1500;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    // Configure pins for SPI
    let sck = gpioc
        .pc10
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
    let miso = gpioc
        .pc11
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);
    let mosi = gpioc
        .pc12
        .into_af_push_pull(&mut gpioc.moder, &mut gpioc.otyper, &mut gpioc.afrh);

    let cs = gpioe.pe8.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade();
    let reset = gpioe.pe9.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade();
    let en_wake = gpioe.pe10.into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper).downgrade();
    let irq = gpioe.pe11.into_pull_up_input(&mut gpioe.moder, &mut gpioe.pupdr).downgrade();

    let spi = Spi::new(dp.SPI3, (sck, miso, mosi), 16.MHz(), clocks, &mut rcc.apb1);
    let delay = Delay::new(cp.SYST, clocks);
    let atwinc1500 = Atwinc1500::new(spi, delay, cs, irq, reset, en_wake, false);

    match atwinc1500 {
        Ok(mut at) => {
            // Get and print the version of the firmware
            // running on the Atwinc1500
            if let Ok(fw) = at.get_firmware_version() {
                info!("Firmware Version: {}", fw);
            }
             
            // Get and print the mac address
            // of the Atwinc1500
            if let Ok(mac) = at.get_mac_address() {
                info!("Mac Address: {}", mac);
            }
        }
        Err(e) => info!("{}", e),
    }

    loop {
        asm::wfi();
    }
}
