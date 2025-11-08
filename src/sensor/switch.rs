use rppal::gpio::Gpio;

/// DC直流开关封装对象
pub struct Switch {
    pin: rppal::gpio::OutputPin,
}

impl Switch {
    /// 创建LED实例
    pub fn new(pin: u8) -> anyhow::Result<Self> {
        // 构建针脚GPIO对象
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin)?.into_output();
        // OK
        Ok(Self { pin })
    }

    /// 闭合开关
    pub fn on(&mut self) {
        if self.pin.is_set_low() {
            self.pin.set_high();
        }
    }

    /// 断开开关
    pub fn off(&mut self) {
        if self.pin.is_set_high() {
            self.pin.set_low()
        }
    }
}
