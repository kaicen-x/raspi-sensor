// src/optimized_dht11.rs
mod sensor;

use sensor::dht11::DHT11;

/// DHT11传感器单总线接入GPIO针脚
const DHT11_PIN: u8 = 4;

fn main() -> anyhow::Result<()> {
    println!("⚡ 优化版 DHT11 读取程序");
    let mut dht11 = DHT11::new(DHT11_PIN)?;
    
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