#![no_std]
#![no_main]

#[cfg(feature="esp32")]
use esp32_hal as hal;
#[cfg(feature="esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature="esp32s3")]
use esp32s3_hal as hal;
#[cfg(feature="esp32c3")]
use esp32c3_hal as hal;

use hal::{
    adc::{AdcConfig, Attenuation, ADC, ADC2},
    clock::ClockControl,
    peripherals::Peripherals,
    gpio::*,
    prelude::*,
    spi,
    timer::TimerGroup,
    Rtc,
    IO,
    Delay,
};

/* Display and graphics stuff */
use max7219::connectors::PinConnector;
use max7219::MAX7219;
use max7219::DecodeMode;

use esp_max7219_nostd::{prepare_display, show_moving_text_in_loop, remove_gaps_in_display_text, draw_point, clear_with_state};
use esp_max7219_nostd::mappings::SingleDisplayData;

use esp_backtrace as _;
use xtensa_atomic_emulation_trap as _;

#[cfg(feature="xtensa-lx-rt")]
use xtensa_lx_rt::entry;
#[cfg(feature="riscv-rt")]
use riscv_rt::entry;
use esp_println::println;

extern crate alloc;
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;

    extern "C" {
        static mut _heap_start: u32;
    }

    unsafe {
        let heap_start = &_heap_start as *const _ as usize;
        ALLOCATOR.init(heap_start as *mut u8, HEAP_SIZE);
    }
}

#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let mut system = peripherals.DPORT.split();
    let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;
    
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut delay = Delay::new(&clocks);

    let din = io.pins.gpio23.into_push_pull_output();
    let cs = io.pins.gpio15.into_push_pull_output();
    let clk = io.pins.gpio18.into_push_pull_output();

    let mut display = MAX7219::from_pins(7, din, cs, clk).unwrap();
    /* this variable will contain actual configuration of display (which points are lit) */
    let mut display_actual_state : SingleDisplayData = [ 0, 0, 0, 0, 0, 0, 0, 0 ]; 

    prepare_display(&mut display, 7, 0x5);
    show_moving_text_in_loop(
        &mut display, 
        "Hello,Espressif!",
        7, 
        25, 
        2, 
        &mut delay,
    );

    loop {}
}