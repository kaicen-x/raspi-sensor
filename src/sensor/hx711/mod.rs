use rppal::gpio::Gpio;
use std::time::{Duration, Instant};

/// 通道和增益
#[derive(Debug, Clone, Copy)]
pub enum Gain {
    /// 通道A，增益128
    /// - 发送一个脉冲
    ChannelA128 = 1,
    /// 通道B，增益32
    /// - 发送二个脉冲
    ChannelB32 = 2,
    /// 通道A，增益64
    /// - 发送三个脉冲
    ChannelA64 = 3,
}

/// HX711 称重传感器封装对象
pub struct HX711 {
    /// 时钟使用的GPIO针脚，树莓派通常为GPIO2
    /// - 需要注意的是树莓派需要启用I2C接口协议
    clock_pin: rppal::gpio::OutputPin,
    /// 数据使用的GPIO针脚，树莓派通常为GPIO3
    /// - 需要注意的是树莓派需要启用I2C接口协议
    data_pin: rppal::gpio::InputPin,
    /// 通道和增益配置
    /// - 每次读取数据后，第25，26，27个脉冲会返回下一次的通道和增益配置
    gain: Gain,
}

/// 实现传感器操作
impl HX711 {
    /// 自实现等待，使用std::thread::sleep会导致主线程被挂起，引发时许错乱问题，导致数据无法接收成功
    /// 这是由于DHT11严格的时序要求导致的
    #[inline(always)]
    fn wait(duration: Duration) {
        let start = Instant::now();
        while start.elapsed() < duration {
            core::hint::black_box(duration);
        }
    }

    /// 构建传感器实例（单从机通信，I2C引脚将被独占）
    pub fn new(clock_pin: u8, data_pin: u8, gain: Gain) -> anyhow::Result<Self> {
        // 创建GPIO实例
        let gpio = Gpio::new()?;
        // 创建时钟引脚实例,并默认置为低电平
        let clock = gpio.get(clock_pin)?.into_output_low();
        // 创建数据引脚实例，并默认为上拉模式
        let data = gpio.get(data_pin)?.into_input_pullup();
        // OK
        Ok(Self {
            clock_pin: clock,
            data_pin: data,
            gain,
        })
    }

    /// 检查HX711 ADC芯片是否就绪
    pub fn is_ready(&self) -> bool {
        // 当DATA引脚为高电平时，表示数据未就绪
        // 一旦为低电平，表示数据就绪，可以读取数据
        self.data_pin.is_low()
    }

    /// 读取HX711输出的数据
    ///
    /// - HX711输出的时24位的数据，所以int32类型足够存储
    pub fn read(&mut self) -> anyhow::Result<i32> {
        // 检查数模转换芯片是否就绪
        if !self.is_ready() {
            return Err(anyhow::anyhow!("HX711数模转换芯片未就绪，请稍后再试"));
        }

        // 读取到的原始数据
        let mut raw_data: u32 = 0;

        // 读取24位数据
        for _ in 0..24 {
            // 发送时钟信号高电平，表示要开始读取一位数据
            self.clock_pin.set_high();
            // 维持高电平信号1微秒能保证时钟信号到达
            Self::wait(Duration::from_micros(1));

            // 读取数据引脚的电平
            if self.data_pin.is_high() {
                // 高电平表示读取到的二进制位为1
                // 把原来的数据左移一位，然后将末尾一位置为1
                raw_data = (raw_data << 1) | 1
            } else {
                // 低电平表示读取到的二进制位为0
                // 把原来的数据左移一位，末尾一位自动就变为0了
                raw_data = raw_data << 1
            }

            // 发送时钟信号低电平，表示读取完一位数据
            self.clock_pin.set_low();
            // 维持低电平信号1微秒能保证时钟信号到达
            Self::wait(Duration::from_micros(1));
        }

        // 设置通道和增益
        // 告知HX711下一次应该发送哪一个通道（A、B两个通道）的数据，并且增益（A支持128和64，B只支持32）是多少.
        // A通道增益128: 发送一个脉冲
        // B通道增益32: 发送二个脉冲
        // A通道增益64: 发送三个脉冲
        for _ in 0..(self.gain as u8) {
            // 发送时钟信号高电平
            self.clock_pin.set_high();
            // 维持高电平信号1微秒能保证时钟信号到达
            Self::wait(Duration::from_micros(1));
            // 发送时钟信号低电平
            self.clock_pin.set_low();
            // 维持高电平信号1微秒能保证时钟信号到达
            Self::wait(Duration::from_micros(1));
        }

        // 确保我们只处理低24位，屏蔽掉可能的高8位
        // 0x00FFFFFF 是 0000 0000 1111 1111 1111 1111 1111 1111
        raw_data &= 0x00FFFFFF;
        // 检查符号位（最高位）
        if (raw_data & 0x00800000) != 0 {
            // 0x00800000 是 0000 0000 1000 0000 0000 0000 0000 0000
            // 如果符号位是1（负数），则进行符号扩展（将高8位置1）
            // 0xFF000000 是 1111 1111 0000 0000 0000 0000 0000 0000
            raw_data = raw_data | 0xFF000000;
        } else {
            // 如果符号位是0（正数），高8位已经是0，无需操作。
            // 或者可以显式地确保高8位为0
            // 0x00FFFFFF 是 0000 0000 1111 1111 1111 1111 1111 1111
            // rawData &= 0x00FFFFFF;
        }

        // OK
        Ok(raw_data as i32)
    }

    /// 重置HX711
    pub fn reset(&mut self) {
        // 时钟引脚保持60微秒即可使HX711芯片断电
        self.clock_pin.set_high();
        Self::wait(Duration::from_micros(60));
        // 60微秒后将时钟信号设为低电平，HX711重新上电，
        self.clock_pin.set_low();
        // 等待1毫秒，以保证时钟引脚处于低电平状态
        Self::wait(Duration::from_millis(1));
    }

    /// 设置HX711的传感器采集通道及增益
    ///
    /// - 设置通道和增益后，根据厂家的文档描述，需要采集4次以上新的数据才会稳定
    pub fn set_gain(&mut self, gain: Gain) {
        self.gain = gain;
    }
}
