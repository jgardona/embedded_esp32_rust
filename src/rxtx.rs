use std::time::Instant;

use esp_idf_svc::{hal::delay::FreeRtos, sys::EspError};

type Result<T> = std::result::Result<T, EspError>;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    println!("ESP32 initialization completed!");
    let start = Instant::now();
    
    loop {
        let elapsed = start.elapsed().as_secs_f32();
        println!("Running time: {elapsed:.1}s");
        FreeRtos::delay_ms(1000);
    }
}