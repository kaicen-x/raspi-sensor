use std::thread;
use std::time::Duration;

use raspi_sensor::sensor::button::Button;
use raspi_sensor::sensor::uln2003a::{Direction, StepMode, ULN2003A};

// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// ULN2003A驱动28BYJ-48电机的4相引脚
const ULN2003A_INT1_PIN: u8 = 6;
const ULN2003A_INT2_PIN: u8 = 13;
const ULN2003A_INT3_PIN: u8 = 19;
const ULN2003A_INT4_PIN: u8 = 26;

/// ULN2003A驱动28BYJ-48电机的测试程序
fn main() -> anyhow::Result<()> {
    //  创建Button实例
    let mut button = Button::new(BUTTON_PIN)?;
    // 创建步进电机实例
    let mut ula2003a = ULN2003A::new(
        ULN2003A_INT1_PIN,
        ULN2003A_INT2_PIN,
        ULN2003A_INT3_PIN,
        ULN2003A_INT4_PIN,
        StepMode::HalfStep,
    )?;

    let mut state = false;
    // 监听按钮状态中断信号
    button.on_change(move |btn_state| {
        // 假设True为按钮按下
        if btn_state {
            // 检测缓存状态
            if !state {
                ula2003a.run_steps(1000, Duration::from_millis(5), Direction::Clockwise);
                println!("检测到按钮按下，顺时针旋转8步")
            } else {
                ula2003a.run_steps(1500, Duration::from_millis(5), Direction::CounterClockwise);
                println!("检测到按钮按下，逆时针旋转10步")
            }
            state = !state;
        }
    })?;

    // 防止程序退出
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}
