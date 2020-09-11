#![no_main]
#![no_std]

use panic_halt;

use cortex_m;
use cortex_m_rt::entry;
use stm32l0xx_hal::{
    pac,
    prelude::*,
    rcc::{Config,MSIRange},
    serial,
};

use ssd1306::{prelude::*, Builder as SSD1306Builder};

use embedded_graphics::{
    fonts::{Font8x16, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    style::TextStyleBuilder,
    };

use core::fmt;
use core::fmt::Write;   
use arrayvec::ArrayString;

use shared_bus;

use vl53l0x::VL53L0x;

const BOOT_DELAY_MS: u16 = 100; 

#[entry]
fn main() -> ! {

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    //configure the clock    
    let mut rcc = dp.RCC.freeze(Config::msi(MSIRange::Range5)); //around 2 MHz, the lowest that actually works here

    //get the delay provider
    let mut delay = cp.SYST.delay(rcc.clocks);

    //delay necessary for the I2C to initiate correctly and start on boot without having to reset the board
    delay.delay_ms(BOOT_DELAY_MS);

    //acquire GPIOA and GPIOB
    let gpioa = dp.GPIOA.split(&mut rcc);
    
    let mut gpiob = dp.GPIOB.split(&mut rcc);

    //set up I2C
    let scl = gpioa.pa9.into_open_drain_output();
    let sda = gpioa.pa10.into_open_drain_output();
    
    let mut i2c = dp.I2C1.i2c(sda, scl, 100.khz(), &mut rcc);

    let manager = shared_bus::CortexMBusManager::new(i2c);
    
    // configure built-in LED
    let mut led = gpiob.pb3.into_push_pull_output();

    // initialize Time of Flight sensor
    let mut tof = VL53L0x::new(manager.acquire()).unwrap();
    
    // initialize OLED in Terminal Mode    
    
    let mut disp: TerminalMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(manager.acquire()).into();

    disp.init().unwrap();
    disp.clear().unwrap();
        
    let mut msg: &str = "                ";

    loop {
        
        let dist = VL53L0x::read_range_single_millimeters_blocking(&mut tof).unwrap();

        // turn LED on when distance less than 100 mm and display a warning message
        if dist < 100 {
            led.set_high().unwrap();
            msg = "Too close!      ";
            }
        
        else {
            led.set_low().unwrap();
            msg = "                ";
            }
        
        let mut buffer = ArrayString::<[u8; 64]>::new();
        
        format(&mut buffer, dist, msg);
            
        disp.write_str(buffer.as_str()).unwrap();
        
        delay.delay_ms(50_u16);

    }

}

fn format(buf: &mut ArrayString<[u8; 64]>, dist: u16, msg: &str) {
    fmt::write(buf, format_args!("Dist:    {:04} mm                {}                ", dist, msg)).unwrap();
}