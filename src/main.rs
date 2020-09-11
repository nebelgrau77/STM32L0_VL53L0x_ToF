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

use core::fmt::Write;
//use nb::block;

use vl53l0x::VL53L0x;

const BOOT_DELAY_MS: u16 = 100; 

#[entry]
fn main() -> ! {

    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    //configure the clock
    // let mut rcc = dp.RCC.freeze(Config::hsi16());
    let mut rcc = dp.RCC.freeze(Config::msi(MSIRange::Range6)); //around 4 MHz, the lowest that actually works here

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
    
    let mut i2c = dp.I2C1.i2c(sda, scl, 400.khz(), &mut rcc);

    //choose RX/TX pins
    let tx_pin = gpioa.pa2;
    let rx_pin = gpioa.pa3;

    //configure serial (default config is 9600 bps)
    let mut serial = dp.USART2.usart(tx_pin, rx_pin, serial::Config::default().baudrate(9600.bps()), &mut rcc).unwrap(); 
   
    let (mut tx, mut _rx) = serial.split();
    
    // configure built-in LED
    let mut led = gpiob.pb3.into_push_pull_output();

    // initialize Time of Flight sensor
    let mut tof = VL53L0x::new(i2c).unwrap();

    //VL53L0x::start_continuous(&mut tof, 50).unwrap();

    loop {
        
        let dist = VL53L0x::read_range_single_millimeters_blocking(&mut tof).unwrap();
        //let dist = VL53L0x::read_range_continuous_millimeters_blocking(&mut tof).unwrap();
        
        // turn LED on when distance less than 100 mm
        if dist < 100 {
            led.set_high().unwrap();
        }
        else {
            led.set_low().unwrap();
        }

        writeln!(tx, "distance: {} mm\r", dist).unwrap();

        delay.delay_ms(50_u16);

    }

}



