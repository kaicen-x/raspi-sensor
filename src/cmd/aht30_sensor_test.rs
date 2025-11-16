use std::{thread, time::Duration};

use raspi_sensor::std_clock::StdClock;
use rppal::i2c::I2c;
use sensor_hal::aht30;

/// AHT30传感器测试程序
fn main() -> anyhow::Result<()> {
    // 初始化全局时钟
    let clock = StdClock::new();
    // 初始化I2C通信总线
    let mut i2c_bus = I2c::new()?;

    // 创建AHT30传感器实例
    let mut aht30_driver = aht30::Driver::new(&clock, &mut i2c_bus, Some(0x38))?;

    // 死循环读取传感器数据
    loop {
        // 读取数据
        match aht30_driver.read(&mut i2c_bus) {
            // 读取成功
            Ok((temperature, humidity)) => {
                println!("读取到的温度: {:.1}℃, 湿度: {:.1}%", temperature, humidity);
            }
            // 读取失败
            Err(err) => {
                eprintln!("读取AHT30传感器温度、湿度失败: {}", err);
            }
        }
        // 间隔100ms读取一次
        thread::sleep(Duration::from_millis(100));
    }
}
