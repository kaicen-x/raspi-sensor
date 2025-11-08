use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use raspi_sensor::sensor::aht30::AHT30;
use rppal::i2c::I2c;

/// AHT30传感器测试程序
fn main() -> anyhow::Result<()> {
    // 初始化I2C通信总线
    let i2c_handle = I2c::new()?;
    // 创建AHT30传感器实例
    let aht30 = AHT30::new(Arc::new(Mutex::new(i2c_handle)), 0x38)?;

    // 死循环读取传感器数据
    loop {
        // 读取数据
        match aht30.read() {
            // 读取成功
            Ok((temperature, humidity)) => {
                println!("读取到的温度: {:.2}℃, 湿度: {:.2}%", temperature, humidity);
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
