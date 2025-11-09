use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use raspi_sensor::sensor::{aht30::AHT30, bme280::BME280};
use rppal::i2c::I2c;

/// AHT30传感器测试程序
fn main() -> anyhow::Result<()> {
    // 初始化I2C通信总线
    let i2c_handle = Arc::new(Mutex::new(I2c::new()?));
    // 创建AHT30传感器实例
    let aht30 = AHT30::new(i2c_handle.clone(), 0x38)?;
    // 创建AHT30传感器实例
    let mut bme280 = BME280::new(i2c_handle.clone(), 0x76)?;

    // 死循环读取传感器数据
    loop {
        // 读取AHT30数据
        match aht30.read() {
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
        match bme280.read() {
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
