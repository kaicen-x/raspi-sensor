// src/optimized_dht11.rs
mod sensor;

use sensor::button::Button;
use sensor::dht11::DHT11;
use sensor::led::LED;

/// DHT11传感器单总线接入GPIO针脚
const DHT11_PIN: u8 = 4;
// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;

fn main() -> anyhow::Result<()> {
    println!("⚡ 优化版 DHT11 读取程序");
    // 创建DHT11传感器实例
    let mut dht11 = DHT11::new(DHT11_PIN)?;
    // 创建Button实例
    let mut button = Button::new(BUTTON_PIN)?;
    // 创建LED实例
    let mut led = LED::new(LED_PIN)?;

    // 监听按钮状态变化
    button.on_change(move |state| {
        if state {
            println!("按钮按下");
            led.open();
        } else {
            println!("按钮松开");
            led.close();
        }
    })?;

    // 死循环读取传感器
    loop {
        match dht11.read() {
            Ok((temp, hum)) => {
                println!("✅ 温度: {:.1}°C, 湿度: {:.1}%", temp, hum);
            }
            Err(e) => {
                eprintln!("❌ 读取失败: {}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
