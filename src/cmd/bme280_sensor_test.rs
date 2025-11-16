use std::{thread, time::Duration};

use raspi_sensor::std_clock::StdClock;
use rppal::i2c::I2c;
use sensor_hal::{aht30, bme280};

/// BME280传感器测试程序
fn main() -> anyhow::Result<()> {
    // 初始化全局时钟
    let clock = StdClock::new();
    // 初始化I2C通信总线
    let mut i2c_bus = I2c::new()?;

    // 创建AHT30传感器实例
    let mut aht30_driver = aht30::Driver::new(&clock, &mut i2c_bus, Some(0x38))?;
    // 创建AHT30传感器实例
    let mut bme280_driver = bme280::Driver::new(&clock, &mut i2c_bus, Some(0x76))?;

    // 死循环读取传感器数据
    loop {
        // 读取AHT30数据
        match aht30_driver.read(&mut i2c_bus) {
            // 读取成功
            Ok((temperature, humidity)) => {
                println!(
                    "AHT30读取到的温度: {:.2}℃, 湿度: {:.2}%",
                    temperature, humidity
                );
            }
            // 读取失败
            Err(err) => {
                eprintln!("读取AHT30传感器温度、湿度失败: {}", err);
            }
        }

        // 读取BME280数据
        match bme280_driver.read(&mut i2c_bus) {
            // 读取成功
            Ok((temperature, pressure, humidity)) => {
                println!(
                    "BME280读取到的温度: {:.2}℃, 压力: {:.2}Pa, 湿度: {:.2}%",
                    temperature, pressure, humidity
                );
            }
            // 读取失败
            Err(err) => {
                eprintln!("读取BME280传感器温度、湿度失败: {}", err);
            }
        }

        // 间隔100ms读取一次
        thread::sleep(Duration::from_millis(1000));
    }
}
