use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicI32, AtomicU32, Ordering},
        mpsc,
    },
    time::Instant,
};
use std::{thread, time::Duration};

use raspi_sensor::{sensor::button::Button, std_clock::StdClock};
use rppal::gpio::{Gpio, InputPin, OutputPin};
use sensor_hal::hx711;

/// 重量状态
#[repr(i32)]
#[derive(Debug)]
pub enum WeightStatus {
    /// 不稳定
    Unstable = 0,
    /// 稳定
    Stable = 1,
    /// 欠载
    Underload = 2,
    /// 超载
    Overload = 3,
    /// 错误
    Error = 4,
}

/// 称重处理器
struct WeightProcessor {
    /// ADC读数最新平均值
    adc_data_latest_average: Arc<AtomicI32>,
    /// ADC读数0点偏移值（俗称皮重）
    adc_data_zero_offset: Arc<AtomicI32>,
    /// ADC读数转换为实物重量时的矫正因子(实际为float32类型)（不受重量单位限制）
    adc_data_transform_factor: Arc<AtomicU32>,
}

/// 实现称重处理器操作
impl WeightProcessor {
    /// 限容队列添加数据
    #[inline(always)]
    fn queue_push<T>(queue: &mut VecDeque<T>, cap: usize, value: T) {
        if queue.len() >= cap {
            // 移除无效的数据
            for _ in 0..(queue.len() - cap + 1) {
                queue.pop_front();
            }
        }
        // 追加最新的数据
        queue.push_back(value);
    }

    /// 计算队列的平均值
    #[inline(always)]
    fn queue_average(queue: &VecDeque<i32>) -> i32 {
        if queue.len() > 0 {
            // 计算缓冲队列的平均值(ADC读数)
            let sum: i32 = queue.iter().sum();
            sum / queue.len() as i32
        } else {
            0
        }
    }

    /// ADC读数转换函数，转换后可得到实际物品的重量
    #[inline(always)]
    fn adc_data_transform(
        adc_data: i32,
        zero_offset: i32,
        transform_factor: f32,
    ) -> anyhow::Result<i32> {
        // 检查矫正因子
        if transform_factor != 0.0 {
            // 计算有效ADC读数
            // 有效ADC读数 = ADC读数 - ADC读数0点偏移值
            let valid_adc_data = adc_data - zero_offset;
            // 计算实际物体的重量
            // 实际物体重量 = 有效ADC读数 / 矫正因子
            let weight = valid_adc_data as f32 / transform_factor;
            // 四舍五入一下得到整数
            Ok(weight.round() as i32)
        } else {
            // 矫正因子为0时无法转换重量
            Err(anyhow::anyhow!("矫正因子为0时无法转换重量，请设置矫正因子"))
        }
    }

    /// 检查当前重量是否稳定
    #[inline(always)]
    fn is_stable(
        adc_data_stable_queue: &VecDeque<i32>,
        sq_cap: usize,
        zero_offset: i32,
        transform_factor: f32,
    ) -> bool {
        if adc_data_stable_queue.len() < sq_cap {
            // 不稳定
            false
        } else {
            // 比较重量（全部一致才认为稳定，注意：不能用ADC读数直接比较）
            let mut tmp_weight: Option<i32> = None;
            for item in adc_data_stable_queue.iter() {
                // 换算为实际物品的重量
                let item_weight =
                    match WeightProcessor::adc_data_transform(*item, zero_offset, transform_factor)
                    {
                        // 重量转换成功
                        Ok(res) => res,
                        // 重量转换失败
                        Err(err) => {
                            // 转换矫正因子为0时直接返回不稳定
                            eprintln!("检查稳定状态失败: {}", err);
                            return false;
                        }
                    };

                // 是否可比较
                match tmp_weight {
                    // 可比较
                    Some(tmp) => {
                        if item_weight != tmp {
                            // 响应不稳定
                            return false;
                        }
                    }
                    // 不可比较
                    None => tmp_weight = Some(item_weight),
                }
            }

            // 默认返回稳定
            return true;
        }
    }

    /// 构建称重处理器实例
    ///
    /// - bq_cap: ADC读数缓冲队列容量
    /// - sq_cap: ADC读数稳定检查队列容量
    pub fn new(
        clock_pin: u8,
        data_pin: u8,
        channel_gain: hx711::ChannelGain,
        bq_cap: usize,
        sq_cap: usize,
        transform_factor: u32,
        sender: mpsc::SyncSender<(i32, WeightStatus)>,
    ) -> anyhow::Result<Self> {
        // 创建GPIO实例
        let gpio = Gpio::new()?;
        let clock: &'static StdClock = Box::leak(Box::new(StdClock::new()));

        // 创建时钟引脚实例,并默认置为低电平
        let clock_gpio = gpio.get(clock_pin)?.into_output_low();
        // 创建数据引脚实例，并默认为上拉模式
        let data_gpio = gpio.get(data_pin)?.into_input_pullup();

        // 构建HX711数模转换传感器实例
        let mut hx711_driver = hx711::Driver::new(clock, clock_gpio, data_gpio, channel_gain)?;

        // ADC读数缓冲队列
        let mut adc_data_buffer_queue: VecDeque<i32> = VecDeque::with_capacity(bq_cap);
        // ADC读数稳定检查队列（存放ADC读数缓冲队列每次更新后的平均值）
        let mut adc_data_stable_queue: VecDeque<i32> = VecDeque::with_capacity(sq_cap);

        // 读取10次有效ADC读数（确保缓冲队列有值，以实现开机去皮）
        for _ in 0..10 {
            // 读取数据
            if let Ok(data) = hx711_driver.read() {
                // 将数据添加到ADC读数缓冲队列
                Self::queue_push(&mut adc_data_buffer_queue, bq_cap, data);
            }
            // 等待100ms, 不然HX711芯片处理不过来
            std::thread::sleep(Duration::from_millis(100));
        }

        // 滤波：计算初始ADC平均读数
        let init_adc_data_average = Self::queue_average(&adc_data_buffer_queue);
        // 将计算得到的ADC平均读数存入稳定检查队列
        Self::queue_push(&mut adc_data_stable_queue, sq_cap, init_adc_data_average);

        // 将计算得到的ADC平均读数存入最新平均读数中
        let adc_data_latest_average = Arc::new(AtomicI32::new(init_adc_data_average));
        // ADC读数0点偏移值（俗称皮重）
        // 将计算得到的ADC平均读数设置为ADC读数0点偏移值，以实现开机去皮
        let adc_data_zero_offset = Arc::new(AtomicI32::new(init_adc_data_average));
        println!("初始皮重(ADC读数): {}", init_adc_data_average);
        // ADC读数转换为实物重量时的矫正因子(实际为float32类型)（不受重量单位限制）
        let adc_data_transform_factor = Arc::new(AtomicU32::new(transform_factor));

        // 克隆一些需要在独立线程中使用的变量
        // 独立线程运行传感器数据读取
        // adc_data_buffer_queue、adc_data_stable_queue不需要克隆，他们的所有权就在独立线程中
        WeightProcessor::loop_read(
            hx711_driver,
            sender,
            adc_data_buffer_queue,
            bq_cap,
            adc_data_stable_queue,
            sq_cap,
            adc_data_latest_average.clone(),
            adc_data_zero_offset.clone(),
            adc_data_transform_factor.clone(),
        );

        // OK
        Ok(Self {
            adc_data_latest_average,
            adc_data_zero_offset,
            adc_data_transform_factor,
        })
    }

    /// 循环读取传感器数据
    fn loop_read(
        mut hx711: hx711::Driver<'static, StdClock, InputPin, OutputPin>,
        sender: mpsc::SyncSender<(i32, WeightStatus)>,
        mut adc_data_buffer_queue: VecDeque<i32>,
        bq_cap: usize,
        mut adc_data_stable_queue: VecDeque<i32>,
        sq_cap: usize,
        adc_data_latest_average: Arc<AtomicI32>,
        adc_data_zero_offset: Arc<AtomicI32>,
        adc_data_transform_factor: Arc<AtomicU32>,
    ) {
        // 异步线程从传感器读取数据
        thread::spawn(move || {
            // 死循环开始读取HX711传感器数据
            loop {
                // 读取数据
                match hx711.read() {
                    // 读取成功
                    Ok(data) => {
                        // 将数据添加到ADC读数缓冲队列
                        WeightProcessor::queue_push(&mut adc_data_buffer_queue, bq_cap, data);
                        // 滤波：计算ADC平均读数
                        let adc_data_average = Self::queue_average(&adc_data_buffer_queue);
                        // 将计算得到的ADC平均读数存入最新平均读数中
                        adc_data_latest_average.store(adc_data_average, Ordering::Release);
                        // 将计算得到的ADC平均读数存入稳定检查队列
                        WeightProcessor::queue_push(
                            &mut adc_data_stable_queue,
                            sq_cap,
                            adc_data_average,
                        );

                        // 提取ADC读数0点偏移值
                        let zero_offset = adc_data_zero_offset.load(Ordering::Acquire);
                        // 提取ADC读数转换矫正因子
                        let transform_factor_u32 =
                            adc_data_transform_factor.load(Ordering::Acquire);
                        let transform_factor = f32::from_bits(transform_factor_u32);

                        // 换算为实际物品的重量
                        let weight = match WeightProcessor::adc_data_transform(
                            adc_data_average,
                            zero_offset,
                            transform_factor,
                        ) {
                            // 重量转换成功
                            Ok(res) => res,
                            // 重量转换失败
                            Err(err) => {
                                eprintln!("转换重量失败: {}", err);
                                // 这个HX711传感器需要间隔100ms读取一次数据
                                thread::sleep(Duration::from_millis(100));
                                continue;
                            }
                        };

                        // 计算状态
                        let weight_status = if weight < 0 {
                            // 欠载
                            WeightStatus::Underload
                        } else if weight > 5000 {
                            // 超载
                            WeightStatus::Overload
                        } else {
                            // 检查是否稳定
                            if WeightProcessor::is_stable(
                                &adc_data_stable_queue,
                                sq_cap,
                                zero_offset,
                                transform_factor,
                            ) {
                                // 稳定
                                WeightStatus::Stable
                            } else {
                                // 不稳定
                                WeightStatus::Unstable
                            }
                        };

                        // 向通道发送数据
                        if let Err(err) = sender.send((weight, weight_status)) {
                            eprintln!("向通道接收者发送读取到的重量失败: {}", err);
                        }
                    }

                    // 读取失败
                    Err(err) => {
                        eprintln!("读取ADC读数失败: {:?}", err);
                        // 向通道发送数据
                        if let Err(err) = sender.send((0, WeightStatus::Error)) {
                            eprintln!("向通道接收者发送异常信息失败: {}", err);
                        }
                    }
                }

                // 这个HX711传感器需要间隔100ms读取一次数据
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    /// 设置皮重
    pub fn set_tare_weight(&self) {
        // 获取当前最新的ADC平均读数
        let adc_data_latest_average = self.adc_data_latest_average.load(Ordering::Acquire);
        // 使用最新的ADC平均读数作为ADC读数0点偏移值
        self.adc_data_zero_offset
            .store(adc_data_latest_average, Ordering::Release);
    }

    /// 设置重量转换因子
    pub fn set_transform_factor(&self, actual_weight: i32) -> anyhow::Result<u32> {
        // 实际重量不能为0，否则无法计算转换因子
        if actual_weight != 0 {
            // 获取当前最新的ADC平均读数
            let adc_data_latest_average = self.adc_data_latest_average.load(Ordering::Acquire);
            // 获取ADC读数0点偏移值
            let adc_data_zero_offset = self.adc_data_zero_offset.load(Ordering::Acquire);
            // 计算有效ADC读数
            // 有效ADC读数 = ADC平均读数 - ADC读数0点偏移值
            let valid_adc_data = adc_data_latest_average - adc_data_zero_offset;
            // 计算转换因子
            // 转换因子 = 有效ADC读数 / 实际重量
            let transform_factor = valid_adc_data as f32 / actual_weight as f32;
            // 将其转换为二进制，以便于通过原子操作存储
            let transform_factor_u32 = transform_factor.to_bits();
            // 保存转换因子
            self.adc_data_transform_factor
                .store(transform_factor_u32, Ordering::Release);
            // 返回计算好的转换因子
            Ok(transform_factor_u32)
        } else {
            Err(anyhow::anyhow!("实际重量不能为0"))
        }
    }
}

// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// HX711传感器接入GPIO针脚
const HX711_DATA_PIN: u8 = 23;
const HX711_CLOCK_PIN: u8 = 24;

/// 称重传感器测试程序
fn main() -> anyhow::Result<()> {
    // 创建按钮实例
    let mut button_driver = Button::new(BUTTON_PIN)?;
    // 创建重量传输通道
    let (weight_sender, weight_reciver) = mpsc::sync_channel::<(i32, WeightStatus)>(1);
    // 转换因子（通常需要持久化存储）
    let transform_factor = (429.58_f32).to_bits();
    // 创建称重处理器实例
    let weight_processor = WeightProcessor::new(
        HX711_CLOCK_PIN,
        HX711_DATA_PIN,
        hx711::ChannelGain::ChannelA128,
        5,
        3,
        transform_factor,
        weight_sender,
    )?;

    // 监听按钮状态变化
    // 实现短按去皮（3秒以内）、长按矫正（3秒以上）
    // 记录按下的时间点
    let mut down_time = Instant::now();
    button_driver.on_change(move |state| {
        // 当按钮按下时执行去皮
        if state {
            // 记录按下的时间点
            down_time = Instant::now();
        } else {
            // 按键松开
            // 计算距离按下的时间点已经过了多少时间
            let duration = down_time.elapsed();
            if duration > Duration::from_secs(3) {
                // 执行矫正
                // TODO: 这里假设放置在秤盘上的砝码是100g，如果是其他重量按需修改即可
                // 包括重量单位也是通过矫正因子直接转换的，比如放了100g的砝码，这里的实际重量传入100000毫克，则最后输出的重量就是以毫克为单位
                // 不过像HX711数模转换芯片搭配的称架一般精度最多只能到克了，干扰大会导致小重量乱跳
                match weight_processor.set_transform_factor(100) {
                    Ok(transform_factor) => {
                        println!(
                            "设置转换矫正因子成功, 当前矫正因子: {}",
                            f32::from_bits(transform_factor)
                        );
                    }
                    Err(err) => {
                        eprintln!("设置转换矫正因子失败: {}", err);
                    }
                }
            } else {
                // 执行去皮
                weight_processor.set_tare_weight();
            }
        }
    })?;

    // 循环显示重量
    loop {
        // 接收称重处理器传出的数据
        match weight_reciver.recv() {
            Ok((weight, status)) => {
                println!("读取到重量: {}g, 状态: {:?}", weight, status);
            }

            // 接收重量数据失败
            Err(err) => {
                eprintln!("重量传输通道接收数据失败: {}", err);
            }
        }
    }
}
