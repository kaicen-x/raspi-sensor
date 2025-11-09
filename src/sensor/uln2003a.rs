use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

/// 步进电机转动方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// 顺时针方向
    Clockwise,
    /// 逆时针方向
    CounterClockwise,
}

/// 步进模式枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepMode {
    WaveDrive, // 单相激励（4步）
    FullStep,  // 双相激励（4步）
    HalfStep,  // 单双相交替（8步）
}

/// ULN2003A驱动模块28BYJ-48电机封装对象
pub struct ULN2003A {
    /// 引脚列表
    pins: [OutputPin; 4],
    /// 当前步进模式
    step_mode: StepMode,
    /// 当前步进序列
    step_sequence: Vec<[bool; 4]>,
    /// 当前步
    current_step: usize,
}

impl ULN2003A {
    /// 生成步进序列
    fn generate_step_sequence(mode: StepMode) -> Vec<[bool; 4]> {
        match mode {
            StepMode::WaveDrive => {
                // 单相激励序列（4步）
                vec![
                    [true, false, false, false], // A
                    [false, true, false, false], // B
                    [false, false, true, false], // C
                    [false, false, false, true], // D
                ]
            }
            StepMode::FullStep => {
                // 双相激励序列（4步）
                vec![
                    [true, true, false, false], // AB
                    [false, true, true, false], // BC
                    [false, false, true, true], // CD
                    [true, false, false, true], // DA
                ]
            }
            StepMode::HalfStep => {
                // 单双相交替序列（8步）- 提供更平滑的运动
                vec![
                    [true, false, false, false], // A
                    [true, true, false, false],  // AB
                    [false, true, false, false], // B
                    [false, true, true, false],  // BC
                    [false, false, true, false], // C
                    [false, false, true, true],  // CD
                    [false, false, false, true], // D
                    [true, false, false, true],  // DA
                ]
            }
        }
    }

    /// 创建新的步进电机实例
    pub fn new(pin1: u8, pin2: u8, pin3: u8, pin4: u8, mode: StepMode) -> anyhow::Result<Self> {
        // 创建GPIO对象
        let gpio = Gpio::new()?;

        // 构建GPIO引脚对象列表
        let pins = [
            gpio.get(pin1)?.into_output_low(),
            gpio.get(pin2)?.into_output_low(),
            gpio.get(pin3)?.into_output_low(),
            gpio.get(pin4)?.into_output_low(),
        ];

        // 根据步进模式生成步进序列
        let step_sequence = Self::generate_step_sequence(mode);

        // OK
        Ok(Self {
            pins,
            step_mode: mode,
            step_sequence,
            current_step: 0,
        })
    }

    /// 设置步进模式
    pub fn set_step_mode(&mut self, mode: StepMode) {
        if mode != self.step_mode {
            self.step_mode = mode;
            self.step_sequence = Self::generate_step_sequence(mode);
            self.current_step = 0;
        }
    }

    /// 应用当前步进序列到GPIO引脚
    fn apply_step(&mut self) {
        let current_pattern = &self.step_sequence[self.current_step];

        for (i, &enabled) in current_pattern.iter().enumerate() {
            if enabled {
                self.pins[i].set_high();
            } else {
                self.pins[i].set_low();
            }
        }
    }

    /// 单步运行
    /// 
    /// - 28BYJ-48建议每步之间的间隔时间最小为3毫秒
    pub fn step(&mut self, direction: Direction) {
        let seq_len = self.step_sequence.len();

        match direction {
            Direction::Clockwise => {
                self.current_step = (self.current_step + 1) % seq_len;
            }
            Direction::CounterClockwise => {
                self.current_step = if self.current_step == 0 {
                    seq_len - 1
                } else {
                    self.current_step - 1
                };
            }
        }

        self.apply_step();
    }

    /// 运行指定步数
    /// 
    /// - steps: 需要步进的步数
    /// - step_delay: 每步之间的间隔时间，28BYJ-48建议最小为3毫秒，该函数限制最小值为3毫秒
    /// - direction: 电机旋转方向
    pub fn run_steps(&mut self, steps: i32, step_delay: Duration, direction: Direction) {
        let step_count = steps.abs() as usize;

        for _ in 0..step_count {
            self.step(direction);
            // 确保最小步间延迟，否则丢步
            thread::sleep(step_delay.max(Duration::from_millis(3)));
        }
    }

    /// 释放电机（停止所有线圈）
    pub fn release(&mut self) {
        for pin in &mut self.pins {
            pin.set_low();
        }
    }

    /// 获取当前步进位置
    pub fn current_position(&self) -> usize {
        self.current_step
    }

    /// 获取序列长度
    pub fn sequence_length(&self) -> usize {
        self.step_sequence.len()
    }
}
