use std::{thread, time::Duration};

use raspi_sensor::std_clock::StdClock;
use rppal::gpio::{Gpio, Mode};
use sensor_hal::dht11;

/// DHT11传感器单总线接入GPIO针脚
const DHT11_PIN: u8 = 4;

fn main() -> anyhow::Result<()> {
    // 初始化GPIO实例
    let gpio = Gpio::new()?;
    let clock = StdClock::new();

    // 创建DHT11传感器引脚实例
    let dht11_gpio = gpio.get(DHT11_PIN)?.into_io(Mode::Output);
    // 创建DHT11传感器驱动实例
    let mut dht11_driver = dht11::Driver::new(&clock, dht11_gpio)?;

    // 死循环读取传感器
    loop {
        // 读取DHT11传感器数据
        match dht11_driver.read() {
            Ok((temp, hum)) => {
                println!("✅ 温度: {:.1}°C, 湿度: {:.1}%", temp, hum);
            }
            Err(e) => {
                eprintln!("❌ 读取失败: {:?}", e);
            }
        }

        // DHT11芯片必须间隔2秒以上才能读取下一次数据
        thread::sleep(Duration::from_secs(2));
    }
}
