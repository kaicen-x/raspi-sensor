use std::{thread, time::Duration};

use raspi_sensor::sensor::led::LED;

// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;

/// LED灯传感器测试程序
fn main() -> anyhow::Result<()> {
    // 创建LED实例
    let mut led = LED::new(LED_PIN)?;
    // 初始将灯关闭
    led.close();

    // 死循环读取传感器
    loop {
        // 等1秒后打开灯
        thread::sleep(Duration::from_secs(1));
        led.open();
        // 等1秒后关闭灯
        thread::sleep(Duration::from_secs(1));
        led.close();
    }
}
