use esp_idf_svc::{
    hal::{
        adc::{
            attenuation::DB_11,
            oneshot::{
                config::{AdcChannelConfig, Calibration},
                AdcChannelDriver, AdcDriver,
            },
        },
        delay::FreeRtos,
        peripherals::Peripherals,
    },
    sys::EspError,
};
use log::info;

type Result<T> = std::result::Result<T, EspError>;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take()?;

    // Configuramos o gpio34. Ele é um pino analógico válido no esp32 clássico.
    let adc = AdcDriver::new(peripherals.adc1)?;
    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: Calibration::Line,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(&adc, peripherals.pins.gpio34, &config)?;

    loop {
        let adc_val = adc.read(&mut adc_pin)?;
        let voltage = adc_val as f64 / 1000.0;
        info!("ADC Val: {adc_val}, \t Voltage: {voltage:.2}V");
        FreeRtos::delay_ms(200);
    }
}
