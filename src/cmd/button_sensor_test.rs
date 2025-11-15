use std::thread;
use std::time::Duration;

use raspi_sensor::{input_pin_wapper::InputPinWapper, output_pin_wapper::OutputPinWapper};
use rppal::gpio::Gpio;
use sensor_hal::{
    button::{self},
    led,
};

// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;

/// 按键传感器测试程序
fn main() -> anyhow::Result<()> {
    let gpio = Gpio::new()?;

    //  创建Button实例
    let button_gpio = InputPinWapper::new(gpio.get(BUTTON_PIN)?.into_input_pullup());
    let mut button_driver = button::AntishakeDriver::new(button_gpio, button::PinState::Low)?;
    // 创建LED实例
    let led_gpio = OutputPinWapper::new(gpio.get(LED_PIN)?.into_output_low());
    let mut led_driver = led::Driver::new(led_gpio, led::PinState::High);

    // 初始LED灯为关闭状态
    let mut btn_state: bool = false;
    let mut led_state: bool = false;

    // 死循环
    loop {
        // 读取一下按钮状态吧(sensor-hal暂时无法提供中断处理，只能使用同步的方式)
        let state = button_driver.state()?;
        // 按钮按下 且 已经松开过
        if state && !btn_state {
            // 检测LED灯的状态
            if led_state {
                // 关闭LED灯
                led_driver.off()?;
                led_state = false;
                println!("检测到按钮按下，关闭LED灯")
            } else {
                // 打开LED灯
                led_driver.on()?;
                led_state = true;
                println!("检测到按钮按下，打开LED灯")
            }
        }
        // 更新按钮状态
        btn_state = state;
        // 加点间隔减少CPU占用
        thread::sleep(Duration::from_millis(1));
    }
}
