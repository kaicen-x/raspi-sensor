use rppal::gpio::Gpio;

/// LED灯封装对象
pub struct LED {
    pin: rppal::gpio::OutputPin,
}

impl LED {
    /// 创建LED实例
    pub fn new(pin: u8) -> anyhow::Result<Self> {
        // 构建针脚GPIO对象
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin)?.into_output();
        // OK
        Ok(Self { pin })
    }

    /// 开启LED灯
    pub fn open(&mut self) {
        if self.pin.is_set_low() {
            self.pin.set_high();
        }
    }

    /// 关闭LED灯
    pub fn close(&mut self) {
        if self.pin.is_set_high() {
            self.pin.set_low()
        }
    }
}
