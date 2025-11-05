use rppal::gpio::{Gpio, Trigger};
use std::time::Duration;

/// 按钮封装对象
pub struct Button {
    pin: rppal::gpio::InputPin,
}

impl Button {
    /// 创建按钮实例
    pub fn new(pin: u8) -> anyhow::Result<Self> {
        // 构建针脚GPIO对象
        let gpio = Gpio::new()?;
        let pin = gpio.get(pin)?.into_input_pullup();
        // OK
        Ok(Self { pin })
    }

    /// 读取当前按钮状态
    /// 
    /// - True: 表示按钮已按下(松开)
    /// - False: 表示按钮已松开(按下)
    #[allow(unused)]
    pub fn read(&mut self) -> bool {
        self.pin.is_low() // 低电平为True
    }

    /// 监听按钮状态变化
    /// 
    /// - True: 表示按钮已按下(松开)
    /// - False: 表示按钮已松开(按下)
    pub fn on_change<F>(&mut self, mut cb: F) -> anyhow::Result<()>
    where
        F: FnMut(bool) + Send + 'static,
    {
        // 设置中断回调，监听电平变化（按下和松开都监听）
        self.pin.set_async_interrupt(
            // 同时监听上升沿和下降沿
            Trigger::Both,
            // 50ms防抖动,
            Some(Duration::from_millis(50)),
            // 下降沿为True
            move |event| cb(event.trigger == Trigger::FallingEdge),
        )?;
        // OK
        Ok(())
    }
}
