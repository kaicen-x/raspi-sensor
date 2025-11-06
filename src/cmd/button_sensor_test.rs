use std::thread;
use std::time::Duration;

use raspi_sensor::sensor::button::Button;
use raspi_sensor::sensor::led::LED;

// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// LED灯接入GPIO针脚
const LED_PIN: u8 = 27;

/// 按键传感器测试程序
fn main() -> anyhow::Result<()> {
    //  创建Button实例
    let mut button = Button::new(BUTTON_PIN)?;
    // 创建LED实例
    let mut led = LED::new(LED_PIN)?;
    // 初始LED灯为关闭状态
    led.close();
    let mut led_state: bool = false;

    // 监听按钮状态中断信号
    button.on_change(move |btn_state| {
        // 假设True为按钮按下
        if btn_state {
            // 检测LED灯的状态
            if led_state {
                // 关闭LED灯
                led.close();
                led_state = false;
                println!("检测到按钮按下，关闭LED灯")
            } else {
                // 打开LED灯
                led.open();
                led_state = true;
                println!("检测到按钮按下，打开LED灯")
            }
        }
    })?;

    // 死循环
    loop {
        // 找不到操作的，读取一下按钮状态吧
        let btn_state = button.read();
        println!("当前按钮状态: {}", btn_state);
        // 加点间隔减少CPU占用
        thread::sleep(Duration::from_secs(1));
    }
}
