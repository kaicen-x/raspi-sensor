use std::{thread, time::Duration};

use raspi_sensor::{input_pin_wapper::InputPinWapper, output_pin_wapper::OutputPinWapper};
use rppal::gpio::Gpio;
use sensor_hal::{button, led, switch};

// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;
// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// DC直流开关接入GPIO针脚
const SWITCH_PIN: u8 = 22;

/// LED灯传感器测试程序
fn main() -> anyhow::Result<()> {
    let gpio = Gpio::new()?;

    // 创建LED灯实例
    let led_gpio = OutputPinWapper::new(gpio.get(LED_PIN)?.into_output_low());
    let mut led_driver = led::Driver::new(led_gpio, led::PinState::High);
    //  创建Button实例
    let button_gpio = InputPinWapper::new(gpio.get(BUTTON_PIN)?.into_input_pullup());
    let mut button_driver = button::AntishakeDriver::new(button_gpio, button::PinState::Low)?;
    // 创建DC开关实例
    let switch_gpio = OutputPinWapper::new(gpio.get(SWITCH_PIN)?.into_output_low());
    let mut switch_driver = switch::Driver::new(switch_gpio, switch::PinState::High);

    // 开关状态（默认关闭）
    let mut btn_state: bool = false;
    let mut switch_state = false;

    // 死循环防止进程退出
    loop {
        // 读取一下按钮状态吧(sensor-hal暂时无法提供中断处理，只能使用同步的方式)
        let state = button_driver.state()?;
        // 按钮按下 且 已经松开过
        if state && !btn_state {
            // 检测开关的状态
            if switch_state {
                // 处于闭合状态，需要断开
                switch_driver.off()?;
                led_driver.off()?;
            } else {
                // 处于断开状态，需要闭合
                switch_driver.on()?;
                led_driver.on()?;
            }
            // 修改开关状态
            switch_state = !switch_state;
        }
        // 更新按钮状态
        btn_state = state;

        // 等1ms后打开灯
        thread::sleep(Duration::from_millis(1));
    }
}
