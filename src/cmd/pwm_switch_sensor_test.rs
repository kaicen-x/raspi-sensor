use std::{thread, time::Duration};

use raspi_sensor::sensor::pwm_switch::PwmSwitch;

// DC直流开关PWM接入GPIO针脚
const SWITCH_PIN: u8 = 13;

/// LED灯传感器测试程序
fn main() -> anyhow::Result<()> {
    // 创建DC PWM开关实例
    let mut pwm_switch_driver = PwmSwitch::new(SWITCH_PIN)?;

    for i in 1..=100 {
        // 100毫秒后：频率50Hz，占空比10%
        thread::sleep(Duration::from_millis(200));
        pwm_switch_driver.set_pwm_frequency(50.0, i as f64 / 100.0)?;
        println!("{}", i);
    }

    // 死循环防止进程退出
    loop {
        // 等1秒后打开灯
        thread::sleep(Duration::from_secs(1));
    }
}
