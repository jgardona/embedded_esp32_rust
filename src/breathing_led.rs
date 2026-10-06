use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::{
    hal::{ledc::config::TimerConfig, peripherals::Peripherals},
    sys::EspError,
};

type Result<T> = std::result::Result<T, EspError>;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take()?;
    let timer_config = TimerConfig::new().frequency(1.kHz().into());
    let timer = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config)?;
    let mut channel = LedcDriver::new(peripherals.ledc.channel0, timer, peripherals.pins.gpio2)?;
    let max_duty = channel.get_max_duty(); // 255 para 8 bits
    loop {
        // fadein
        for duty in 0..=max_duty {
            channel.set_duty(duty)?;
            FreeRtos::delay_ms(10);
        }
        // fadeout
        for duty in (0..=max_duty).rev() {
            channel.set_duty(duty)?;
            FreeRtos::delay_ms(10);
        }
    }
}
