#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::gpio::Level;

//use dshot::{DShot, DShotSpeed};

use dshot::{DShot, DShotSpeed};



#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.0.1

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    // TODO: Spawn some tasks
    let _ = spawner;


    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let esc_pin = io.pins.gpio18;

    let rmt = Rmt::new(peripherals.RMT, 80.MHz(), &clocks).expect("rtm başlamadı");
    
    let tx_config = esp_hal::rmt::TxChannelConfig::default();
    let channel = rmt.channel0.configure_async(esc_pin, tx_config).expect("chanel olmadı");

    let mut esc = DShot::new(channel, DShotSpeed::DShot600, None, None);

    match esc.arm().await {
        Ok(_) => esp_println::println!("arm oldu"),
        Err(e) => esp_println::println!("arm olmadı: {}", e),
    }

    let mut throttle = 0u16;
    loop {
        if throttle < 500 {
            throttle += 10;
        }

        if let Err(e) = esc.write_throttle(throttle, false).await {
            esp_println::println!("dshot gitmedi: {}", e);
        }

        Timer::after(Duration::from_millis(10)).await;
    }

}

