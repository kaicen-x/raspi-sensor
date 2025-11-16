use std::{thread, time::Duration};

// use raspi_sensor::pwm_wapper::PwmWapper;
use rppal::pwm::{Channel, Pwm};
use sensor_hal::dc_relay;

/// DC直流开关传感器PWM测试程序
fn main() -> anyhow::Result<()> {
    let dc_relay_pwm = Pwm::new(Channel::Pwm0, 1000)?;

    // 创建DC PWM开关实例
    let mut dc_relay_pwm_driver = dc_relay::PwmDriver::new(dc_relay_pwm);

    for i in 1..=100 {
        // 100毫秒后：频率50Hz，占空比10%
        thread::sleep(Duration::from_millis(200));
        dc_relay_pwm_driver.set_duty_cycle_fraction(50, 100)?;
        println!("{}", i);
    }

    // 死循环防止进程退出
    loop {
        // 等1秒
        thread::sleep(Duration::from_secs(1));
    }
}
