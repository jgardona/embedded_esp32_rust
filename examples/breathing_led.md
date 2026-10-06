# Controle PWM

### 1. C++ / Arduino — ESP32 clássico

Diferença principal: em vez de `ledcAttachChannel` (API unificada por pino do core 3.x), uso a API clássica em duas etapas — `ledcSetup` (configura canal/frequência/resolução) + `ledcAttachPin` (liga o pino ao canal) — e `ledcWrite` passa a receber o **canal**, não o pino.

```cpp
/****************************************************************
  Filename    : BreathingLight
  Description : Make led light fade in and out, just like breathing.
  Target      : ESP32 (classic)
 ****************************************************************/
#define PIN_LED 2     //define the led pin
#define CHN     0     //define the pwm channel
#define FRQ     1000  //define the pwm frequency
#define PWM_BIT 8     //define the pwm precision

void setup() {
  ledcSetup(CHN, FRQ, PWM_BIT);       //configure the pwm channel
  ledcAttachPin(PIN_LED, CHN);        //attach the led pin to the channel
}

void loop() {
  for (int i = 0; i < 255; i++) {     //make light fade in
    ledcWrite(CHN, i);                //write duty by CHANNEL, not pin
    delay(10);
  }
  for (int i = 255; i > -1; i--) {    //make light fade out
    ledcWrite(CHN, i);
    delay(10);
  }
}
```

### 2. Rust — ESP32 clássico (esp-idf-hal / std)

Pressupõe um projeto gerado com `cargo generate esp-rs/esp-idf-template` já configurado para o target `xtensa-esp32-espidf` (não estou criando o projeto, só o código conforme combinado).

```rust
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;

    let timer_config = TimerConfig::new().frequency(1.kHz().into());
    let timer = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config)?;

    let mut channel = LedcDriver::new(
        peripherals.ledc.channel0,
        timer,
        peripherals.pins.gpio2,
    )?;

    let max_duty = channel.get_max_duty(); // 255 para 8 bits

    loop {
        // fade in
        for duty in 0..=max_duty {
            channel.set_duty(duty)?;
            FreeRtos::delay_ms(10);
        }
        // fade out
        for duty in (0..=max_duty).rev() {
            channel.set_duty(duty)?;
            FreeRtos::delay_ms(10);
        }
    }
}
```

Detalhes que replicam o original: canal LEDC 0, GPIO2, 1 kHz. A resolução vem automaticamente da configuração padrão do `TimerConfig` (8 bits, `max_duty = 255`), correspondendo ao `PWM_BIT 8` do sketch — se quiser forçar explicitamente, dá pra adicionar `.resolution(Resolution::Bits8)` no `TimerConfig::new()`.

Claro, vou explicar linha por linha o código Rust.

## Imports

```rust
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
```

- `FreeRtos` — dá acesso a `FreeRtos::delay_ms(...)`, o equivalente ao `delay()` do Arduino. O ESP-IDF roda sobre o FreeRTOS, então "esperar" na prática é ceder tempo pro scheduler, não travar a CPU como um `delay` bruto.
- `TimerConfig`, `LedcDriver`, `LedcTimerDriver` — são as três peças do módulo LEDC (o periférico de PWM do ESP32) no `esp-idf-hal`. Isso espelha a separação conceitual `ledcSetup` (timer) + `ledcAttachPin` (canal) do Arduino, só que em Rust cada etapa vira um tipo próprio.
- `Peripherals` — struct que representa **todo o hardware do chip** (pinos, timers, UART, etc). Em Rust/embedded é comum usar o padrão *ownership de periféricos*: você "toma posse" de uma peça de hardware uma única vez, e o compilador garante que ninguém mais pode usá-la ao mesmo tempo (evita, por exemplo, dois pedaços de código configurando o mesmo pino).
- `prelude::*` — importa traits utilitários, como o `.kHz()` que uso mais abaixo para converter um número em frequência.

## `fn main() -> anyhow::Result<()>`

```rust
fn main() -> anyhow::Result<()> {
```

O `main` retorna um `Result`. Em Rust, funções que podem falhar (aqui, qualquer chamada ao ESP-IDF pode falhar — pino ocupado, driver indisponível, etc.) retornam `Result<T, E>`. Uso `anyhow::Result<()>` como um "Result genérico" que aceita qualquer erro, então não preciso tratar cada tipo de erro específico manualmente — se algo falhar, a função simplesmente retorna o erro e o programa encerra com a mensagem.

```rust
esp_idf_svc::sys::link_patches();
```

Isso é um detalhe de "boilerplate" do ESP-IDF em Rust: força o linker a incluir alguns patches necessários do runtime do IDF (sem isso, algumas funções do IDF simplesmente não entram no binário final). Não tem equivalente no Arduino — lá isso é feito automaticamente pelo core.

## Tomando posse dos periféricos

```rust
let peripherals = Peripherals::take()?;
```

Pega a struct única que representa o hardware. O `?` no final é o operador de propagação de erro: se `take()` falhar, a função já retorna o erro ali mesmo (equivale a um `if erro { return Err(...) }` implícito). `take()` só pode ser chamado com sucesso **uma vez** no programa todo — é assim que o Rust garante, em tempo de compilação, que você não vai configurar o mesmo periférico duas vezes em lugares diferentes do código.

## Configurando o timer (equivalente ao `ledcSetup`)

```rust
let timer_config = TimerConfig::new().frequency(1.kHz().into());
let timer = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config)?;
```

- `TimerConfig::new()` cria uma configuração padrão (resolução 8 bits por default) e `.frequency(1.kHz().into())` define 1000 Hz — o `1.kHz()` é o tal do `.kHz()` do prelude, uma forma "legível" de escrever frequências (equivale ao `FRQ 1000` do `#define`).
- `LedcTimerDriver::new(...)` efetivamente configura o timer 0 do periférico LEDC com essa frequência/resolução — isso é o `ledcSetup(CHN, FRQ, PWM_BIT)` do C++, só que separando timer de canal (no ESP-IDF, timer e canal são conceitos distintos: o timer gera a base de tempo/PWM, o canal roteia isso para um pino).

## Configurando o canal + pino (equivalente ao `ledcAttachPin`)

```rust
let mut channel = LedcDriver::new(
    peripherals.ledc.channel0,
    timer,
    peripherals.pins.gpio2,
)?;
```

Cria o driver do canal 0, associando o `timer` configurado acima ao pino físico `gpio2`. Isso substitui o `ledcAttachPin(PIN_LED, CHN)`. Note que `timer` é **movido** para dentro dessa chamada (ownership) — depois dessa linha, você não usa mais a variável `timer` diretamente, ela "pertence" ao `channel` agora.

`mut` é necessário porque, mais abaixo, vou chamar `channel.set_duty(...)`, que modifica o estado interno do driver (escreve no periférico).

## Duty máximo

```rust
let max_duty = channel.get_max_duty();
```

Pergunta ao driver qual é o valor máximo de duty cycle para a resolução configurada. Com 8 bits, isso vale 255 — dá o mesmo efeito do `i < 255` / `i > -1` do C++, mas de forma explícita e sem número mágico: se você mudar a resolução do timer, esse valor se ajusta sozinho.

## O loop (fade in / fade out)

```rust
loop {
    for duty in 0..=max_duty {
        channel.set_duty(duty)?;
        FreeRtos::delay_ms(10);
    }
    for duty in (0..=max_duty).rev() {
        channel.set_duty(duty)?;
        FreeRtos::delay_ms(10);
    }
}
```

- `loop { }` é o `void loop()` do Arduino — repete para sempre.
- `for duty in 0..=max_duty` é um `for` com um **range inclusivo** (`..=` inclui o valor final, diferente de `..` que exclui) — equivale ao primeiro `for (int i = 0; i < 255; i++)`, só que indo até `max_duty` (255) em vez de parar em 254.
- `channel.set_duty(duty)?` escreve o duty cycle no canal PWM — é o `ledcWrite(CHN, i)`. De novo o `?`: se a escrita falhar, propaga o erro.
- `FreeRtos::delay_ms(10)` é o `delay(10)`.
- O segundo `for` usa `.rev()` para inverter o range e ir de `max_duty` até `0` — equivale ao `for (int i = 255; i > -1; i--)` de fade out.

## Resumo da correspondência com o C++

| C++ (ESP32 clássico) | Rust (esp-idf-hal) |
|---|---|
| `ledcSetup(CHN, FRQ, PWM_BIT)` | `TimerConfig` + `LedcTimerDriver::new(...)` |
| `ledcAttachPin(PIN_LED, CHN)` | `LedcDriver::new(channel0, timer, gpio2)` |
| `ledcWrite(CHN, i)` | `channel.set_duty(duty)?` |
| `delay(10)` | `FreeRtos::delay_ms(10)` |
| `for (...)` normal | `for duty in 0..=max_duty` / `.rev()` |
| (implícito, sempre funciona) | `?` — erros são explícitos e propagados |