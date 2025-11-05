use rppal::gpio::Gpio;
use std::time::{Duration, Instant};

/// DHT11 温度、湿度二合一传感器封装对象
pub struct DHT11 {
    /// 使用的GPIO针脚，树莓派通常为GPIO4
    /// 需要注意的是树莓派需要启用1-wire(One-Wire)接口协议
    pin: rppal::gpio::IoPin,
}

/// 实现传感器操作
impl DHT11 {
    /// 自实现等待，使用std::thread::sleep会导致主线程被挂起，引发时许错乱问题，导致数据无法接收成功
    /// 这是由于DHT11严格的时序要求导致的
    fn wait(duration: Duration) {
        let start = Instant::now();
        while start.elapsed() < duration {}
    }

    /// 构建传感器实例
    pub fn new(pin: u8) -> anyhow::Result<Self> {
        // 构建针脚GPIO对象
        let gpio = Gpio::new()?;
        let mut pin = gpio.get(pin)?.into_io(rppal::gpio::Mode::Output);
        // 设置高电平
        pin.set_high();
        // 些许的等待可以让传感器收到高电平信号,使电平稳定
        Self::wait(Duration::from_secs(1));
        // OK
        Ok(Self { pin })
    }

    /// 在指定时间范围内等待一个高(低)电平信号，超过该时间范围就认为是低(高)电平信号
    fn wait_for_edge(&self, target_high: bool, timeout_us: u64) -> bool {
        let start = Instant::now();
        while self.pin.is_high() != target_high {
            if start.elapsed() > Duration::from_micros(timeout_us) {
                return false;
            }
        }
        true
    }

    // 在指定时间范围内等待高电平信号
    fn measure_high_time(&self, timeout_us: u64) -> anyhow::Result<Duration> {
        let start = Instant::now();
        while self.pin.is_high() {
            if start.elapsed() > Duration::from_micros(timeout_us) {
                return Err(anyhow::anyhow!("高电平时间测量超时"));
            }
        }
        Ok(start.elapsed())
    }

    /// 从传感器读取温度和湿度(两次read之间最少间隔2秒，防止传感器过热)
    pub fn read(&mut self) -> anyhow::Result<(f32, f32)> {
        // 发送开始信号（告诉传感器，我要读取数据了，快发给我，别墨迹了）
        self.pin.set_mode(rppal::gpio::Mode::Output);
        self.pin.set_low();
        Self::wait(Duration::from_millis(18));
        self.pin.set_high();
        Self::wait(Duration::from_micros(30));

        // 设置引脚为输入模式
        self.pin.set_mode(rppal::gpio::Mode::Input);

        // 等待低电平（响应开始）
        if !self.wait_for_edge(false, 1000) {
            return Err(anyhow::anyhow!("响应开始超时"));
        }

        // 等待高电平（响应结束）
        if !self.wait_for_edge(true, 1000) {
            return Err(anyhow::anyhow!("响应结束超时"));
        }

        // 等待低电平（数据开始）
        if !self.wait_for_edge(false, 1000) {
            return Err(anyhow::anyhow!("数据开始超时"));
        }

        // 读取40位数据
        // 数据格式：
        // 收到主机信号后，从机一次性从SDA串出40bit，高位先出
        // 8bit湿度整数数据 + 8bit湿度小数数据 + 8bit温度整数数据 + 8bit温度小数数据 + 8bit校验位
        // 高位为温度整数部分，低位为温度小数部分
        // 高位为温度整数部分，低位为湿度小数部分
        // 低位第8位1表示负温度，否则位正温度
        // 校验位=湿度高位+湿度低位+温度高位+温度低位
        let mut data = [0u8; 5];
        for byte in 0..5 {
            for bit in 0..8 {
                if !self.wait_for_edge(true, 1000) {
                    return Err(anyhow::anyhow!("数据位开始超时"));
                }

                let high_time = self.measure_high_time(1000)?;
                if high_time > Duration::from_micros(40) {
                    data[byte] |= 1 << (7 - bit);
                }
            }
        }

        // 校验数据
        let checksum = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);
        if checksum != data[4] {
            return Err(anyhow::anyhow!("校验和错误"));
        }

        // 转换温度湿度为浮点类型
        let humidity = data[0] as f32;
        let temperature = data[2] as f32;

        // OK
        Ok((temperature, humidity))
    }
}
