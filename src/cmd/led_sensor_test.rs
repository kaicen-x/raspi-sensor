use std::{thread, time::Duration};

use raspi_sensor::output_pin_wapper::OutputPinWapper;
use rppal::gpio::Gpio;
use sensor_hal::led;

// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;

/// LED灯传感器测试程序
fn main() -> anyhow::Result<()> {
    let gpio = Gpio::new()?;

    // 创建LED实例
    let led_gpio = OutputPinWapper::new(gpio.get(LED_PIN)?.into_output_low());
    let mut led_driver = led::Driver::new(led_gpio, led::PinState::High);

    // 死循环读取传感器
    loop {
        // 等1秒后打开灯
        thread::sleep(Duration::from_secs(1));
        led_driver.on()?;
        // 等1秒后关闭灯
        thread::sleep(Duration::from_secs(1));
        led_driver.off()?;
    }
}
