use rppal::gpio::Gpio;

/// DC直流PWM开关封装对象
pub struct PwmSwitch {
    pin: rppal::gpio::OutputPin,
}

impl PwmSwitch {
    /// 创建LED实例
    pub fn new(pin: u8) -> anyhow::Result<Self> {
        // 构建针脚GPIO对象
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin)?.into_output_low();

        // OK
        Ok(Self { pin })
    }

    /// 设置PWM频率和占空比
    ///
    /// - 通过占空比调整转速（占空比表示一个周期内的通电时长）
    pub fn set_pwm_frequency(&mut self, frequency: f64, duty_cycle: f64) -> anyhow::Result<()> {
        // 执行设置
        self.pin.set_pwm_frequency(frequency, duty_cycle)?;
        // OK
        Ok(())
    }
}
