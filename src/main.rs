//!
//! Based on an example from https://github.com/jamwaffles/ssd1306
//! Sends random raw data to the display, emulating an old untuned TV. 
//! Retrieves the underlying display properties struct and allows calling of the low-level `draw()` method,
//! sending a 1024 byte buffer straight to the display.
//! 
//! Uses SmallRng as random number generator.
//! NOTE: these are pseudorandom numbers, not suitable for cryptographic or similar purposes.
//! 


#![no_std]
#![no_main]

use arduino_nano33iot as hal;

use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::entry;
use hal::time::KiloHertz;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

use core::fmt;
use core::fmt::Write;   
use arrayvec::ArrayString;

use vl53l0x::VL53L0x;

use ssd1306::{prelude::*, Builder as SSD1306Builder};

use shared_bus;
use arrayvec;

const BOOT_DELAY_MS: u16 = 100; //small delay for the I2C to initiate correctly and start on boot without having to reset the board

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let mut pins = hal::Pins::new(peripherals.PORT);    
    let mut delay = Delay::new(core.SYST, &mut clocks);
    
    let i2c = hal::i2c_master(
        &mut clocks,
        KiloHertz(100),
        peripherals.SERCOM4, 
        &mut peripherals.PM, 
        pins.sda,
        pins.scl,
        &mut pins.port,
    );  

    delay.delay_ms(BOOT_DELAY_MS);

    let manager = shared_bus::CortexMBusManager::new(i2c);

    // configure built-in LED
    let mut led = pins.led_sck.into_open_drain_output(&mut pins.port);

    // initialize Time of Flight sensor    
    let mut tof = VL53L0x::new(manager.acquire()).unwrap();

    // initialize OLED in Terminal Mode    
    let mut disp: TerminalMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(manager.acquire()).into();
    //let mut disp: TerminalMode<_> = SSD1306Builder::new().size(DisplaySize::Display128x32).connect_i2c(i2c).into();

    disp.init().unwrap();
    disp.clear().unwrap();
            
    let mut msg: &str = "                ";

    let mut dist: u16 = 200;

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
        
        //dist -= 1;

        delay.delay_ms(100u16);

    }

}

fn format(buf: &mut ArrayString<[u8; 64]>, dist: u16, msg: &str) {
    fmt::write(buf, format_args!("Dist:    {:04} mm                {}                ", dist, msg)).unwrap();
}




