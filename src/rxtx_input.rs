use std::io::{self, BufRead};

use esp_idf_svc::hal::delay::FreeRtos;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // UART0 (tx0=gpio1, rx0=gpio3) já é usada pelo console do ESP-IDF (log/monitor),
    // então lemos o teclado via stdin em vez de instalar um UartDriver próprio nela
    // (os dois disputariam a mesma UART e corromperiam a entrada/saída).
    println!("ESP32 initialization completed!!");
    println!("Please input some characters:");
    println!("Press <enter> to send message to ESP32.");

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut input_string = String::new();
    loop {
        // stdin é não-bloqueante aqui: sem linha completa disponível, read_line
        // retorna WouldBlock (EAGAIN) em vez de bloquear até um '\n' chegar.
        match stdin.read_line(&mut input_string) {
            Ok(n) if n > 0 => {
                print!("inputString: {input_string}");
                input_string.clear();
            }
            Err(e) if e.kind() != io::ErrorKind::WouldBlock => {
                println!("read error: {e}");
                input_string.clear();
            }
            _ => {}
        }
        FreeRtos::delay_ms(10);
    }
}
