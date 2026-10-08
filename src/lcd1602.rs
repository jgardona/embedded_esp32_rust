use std::time::Instant;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::prelude::*;
use esp_idf_hal::{
    delay::{Ets, BLOCK},
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    sys::{EspError, ESP_FAIL},
};
use hd44780_driver::{Cursor, CursorBlink, Display, DisplayMode, HD44780};

type Result<T> = std::result::Result<T, EspError>;

fn i2c_addr_test(i2c: &mut I2cDriver, addr: u8) -> bool {
    // escrita vazia: retorna ok, se algum dispositivo responder com ACK.
    i2c.write(addr, &[], BLOCK).is_ok()
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sda = peripherals.pins.gpio14;
    let scl = peripherals.pins.gpio13;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    let mut delay = Ets;
    let addr = if i2c_addr_test(&mut i2c, 0x27) {
        0x27
    } else {
        0x3f
    };
    let mut lcd = HD44780::new_i2c(i2c, addr, &mut delay)
        .map_err(|_| EspError::from_infallible::<ESP_FAIL>())?;

    lcd.reset(&mut delay).ok();
    lcd.clear(&mut delay).ok();
    lcd.set_display_mode(
        DisplayMode {
            display: Display::On,
            cursor_visibility: Cursor::Invisible,
            cursor_blink: CursorBlink::Off,
        },
        &mut delay,
    )
    .ok();

    lcd.set_cursor_pos(0, &mut delay).ok();
    lcd.write_str("hello, world!", &mut delay).ok();
    let start = Instant::now();
    loop {
        lcd.set_cursor_pos(0x40, &mut delay).ok();
        let texto = format!("Counter: {}", start.elapsed().as_secs());
        lcd.write_str(&texto, &mut delay).ok();
        FreeRtos::delay_ms(1000);
    }
}
