use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::EspError;
use log::info;

type Result<T> = std::result::Result<T, EspError>;

fn main() -> Result<()> {
    // Necessário para vincular os patches do ESP-IDF (ex: printf, exceções em C++).
    esp_idf_svc::sys::link_patches();
    // Inicializa o logger padrão do ESP-IDF para o log::info!/warn!/etc funcionarem.
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // GPIO2 é o LED onboard mais comum em placas DevKitC/NodeMCU-32S.
    // Troque para o pino correto se sua placa usar outro (ex: gpio5, gpio13...).
    let mut led = PinDriver::output(peripherals.pins.gpio2)?;

    loop {
        led.set_high()?;
        info!("LED ligado");
        FreeRtos::delay_ms(1000);
        led.set_low()?;
        info!("LED desligado");
        FreeRtos::delay_ms(1000);
    }
}
