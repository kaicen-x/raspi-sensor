use std::{thread, time::Duration};

use raspi_sensor::sensor::dht11::DHT11;
use raspi_sensor::sensor::led::LED;

/// DHT11传感器单总线接入GPIO针脚
const DHT11_PIN: u8 = 4;
// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;

fn main() -> anyhow::Result<()> {
    // 创建DHT11传感器实例
    let mut dht11 = DHT11::new(DHT11_PIN)?;
    // 创建LED实例
    let mut led = LED::new(LED_PIN)?;
    // 初始LED灯状态为关闭
    led.close();

    // 死循环读取传感器
    loop {
        // 读取DHT11传感器数据
        match dht11.read() {
            Ok((temp, hum)) => {
                // 温度不在指定范围内需要亮灯
                // 湿度不在指定范围内需要亮灯
                if temp < 10.0 || temp > 40.0 || hum < 20.0 || hum > 60.0 {
                    led.open();
                } else {
                    // 有效范围不需要亮灯
                    led.close();
                }
                println!("✅ 温度: {:.1}°C, 湿度: {:.1}%", temp, hum);
            }
            Err(e) => {
                eprintln!("❌ 读取失败: {}", e);
            }
        }

        // DHT11芯片必须间隔2秒以上才能读取下一次数据
        thread::sleep(Duration::from_secs(2));
    }
}
