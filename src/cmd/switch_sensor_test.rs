use std::{thread, time::Duration};

use raspi_sensor::sensor::{button::Button, led::LED, switch::Switch};

// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;
// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// DC直流开关接入GPIO针脚
const SWITCH_PIN: u8 = 22;

/// LED灯传感器测试程序
fn main() -> anyhow::Result<()> {
    // 创建LED灯实例
    let mut led = LED::new(LED_PIN)?;
    // 创建按钮实例
    let mut button = Button::new(BUTTON_PIN)?;
    // 创建DC开关实例
    let mut switch = Switch::new(SWITCH_PIN)?;

    // 开关状态（默认关闭）
    let mut switch_state = false;

    // 等待按钮按下
    button.on_change(move |state| {
        if state {
            // 按钮被按下
            // 检查开关状态
            if switch_state {
                // 处于闭合状态，需要断开
                switch.off();
                led.close();
            } else {
                // 处于断开状态，需要闭合
                switch.on();
                led.open();
            }
            // 修改开关状态
            switch_state = !switch_state;
        }
    })?;

    // 死循环防止进程退出
    loop {
        // 等1秒后打开灯
        thread::sleep(Duration::from_secs(1));
    }
}
