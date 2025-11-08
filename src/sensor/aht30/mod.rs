use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use rppal::i2c::I2c;

/// AHT30温度湿度传感器封装对象
pub struct AHT30 {
    /// I2C通信句柄
    i2c_handle: Arc<Mutex<I2c>>,
    /// I2C从设备地址
    /// - AHT30的地址通常为: 0x38
    i2c_addr: u8,
}

/// 实现AHT30传感器操作
impl AHT30 {
    /// 创建AHT30传感器实例
    pub fn new(i2c_handle: Arc<Mutex<I2c>>, i2c_addr: u8) -> anyhow::Result<Self> {
        // 1. 上电后等待5ms（文档明确要求）
        thread::sleep(Duration::from_millis(5));

        // 2. 确保锁的最小范围
        {
            // 获取I2C通信句柄操作权限
            let mut i2c_handle_lock = i2c_handle
                .lock()
                .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;

            // 设置从设备地址
            i2c_handle_lock.set_slave_address(i2c_addr as u16)?;

            // 发送传感器初始化命令
            i2c_handle_lock.write(&[0xBE, 0x08, 0x00])?;
        }

        // 等待初始化完成
        thread::sleep(Duration::from_millis(10));
        // 构建传感器实例
        let this = Self {
            i2c_handle,
            i2c_addr,
        };
        // 检查传感器状态
        let status = this.read_status()?;
        if !status.calibration_enabled {
            // 校准功能未启用，则传感器未初始化成功
            return Err(anyhow::anyhow!("传感器初始化失败"));
        }

        // OK
        Ok(this)
    }

    /// 计算CRC8校验和
    fn calc_crc8(data: &[u8]) -> u8 {
        // 声明CRC8校验和结果
        let mut crc8_sum = 0xFF;
        // 遍历处理每一个字节
        for b in data {
            // 当前字节与已经计算的结果进行按位异或运算
            crc8_sum ^= b;
            // 再单独处理每一位二进制
            for _ in 0..8 {
                if crc8_sum & 0x80 != 0 {
                    crc8_sum = (crc8_sum << 1) ^ 0x31;
                } else {
                    crc8_sum <<= 1;
                }
            }
        }
        // OK
        crc8_sum
    }

    /// 读取传感器状态
    pub fn read_status(&self) -> anyhow::Result<Status> {
        // 获取I2C通信句柄操作权限
        let mut i2c_handle_lock = self
            .i2c_handle
            .lock()
            .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;

        // 设置从设备地址
        i2c_handle_lock.set_slave_address(self.i2c_addr as u16)?;

        // 获取传感器状态
        let mut data = [0u8; 1];
        i2c_handle_lock.read(&mut data)?;

        // 解析状态并返回
        Ok(Status::from(data[0]))
    }

    /// 读取传感器数据
    pub fn read(&self) -> anyhow::Result<(f32, f32)> {
        // 获取I2C通信句柄操作权限
        let mut i2c_handle_lock = self
            .i2c_handle
            .lock()
            .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;

        // 设置从设备地址
        i2c_handle_lock.set_slave_address(self.i2c_addr as u16)?;

        // 发送测量命令
        // 文档里写的要发送[0x70, 0xAC, 0x33, 0x00]
        // 0x70是从设备地址加写操作符号，i2c_handle.write内部已自动拼接，无需显式发送
        i2c_handle_lock.write(&[0xAC, 0x33, 0x00])?;

        // 根据文档提示需要等待测量完成（约 80ms）
        thread::sleep(Duration::from_millis(80));

        // 读取7字节数据
        // 第1个字节（8位）: 8位二进制状态位
        // 第2~6个字节（40位）：前20位湿度 + 后20位温度
        // 第7个字节（8位）：CRC8校验字节
        let mut data = [0u8; 7];
        i2c_handle_lock.read(&mut data)?;

        // 对读取到的数据进行CRC校验
        let correct_crc8 = data[6]; // 接收到的正确的CRC8校验值
        let current_crc8 = Self::calc_crc8(&data[0..6]); // 计算出来的CRC8校验值
        if correct_crc8 != current_crc8 {
            return Err(anyhow::anyhow!("校验接收到的CRC8数据失败"));
        }

        // 解析状态信息
        let status = Status::from(data[0]);
        // 检查设备是否繁忙
        if status.is_busy {
            return Err(anyhow::anyhow!("设备繁忙"));
        }

        // 提取 20 位湿度数据
        let humidity_raw =
            ((data[1] as u32) << 12) | ((data[2] as u32) << 4) | ((data[3] as u32) >> 4);

        // 提取 20 位温度数据
        let temperature_raw =
            (((data[3] as u32) & 0b1111) << 16) | ((data[4] as u32) << 8) | data[5] as u32;

        // 转换为实际值
        let humidity = (humidity_raw as f32 / (1u32 << 20) as f32) * 100.0;
        let temperature = (temperature_raw as f32 / (1u32 << 20) as f32) * 200.0 - 50.0;

        // OK
        Ok((temperature, humidity))
    }
}

/// AHT30温度湿度传感器工作模式
#[derive(Debug)]
pub enum WorkingMode {
    /// 正常模式
    NOR,
    /// 循环模式
    CYC,
    /// 命令模式
    CMD,
}

/// AHT30温度湿度传感器状态
///
/// 二进制位从右往左数，例如：0b00000001, 第0位是1
#[derive(Debug)]
pub struct Status {
    /// 二进制位第0位和第1位暂时空置
    _0_1: (),
    /// 校准后的电容数据是否超出CMP中断阈值范围
    ///
    /// 二进制位第2位:
    /// - 0--校准后的电容数据未超出CMP中断阈值范围
    /// - 1--校准后的电容数据超出CMP中断阈值范围
    pub cmp_interrupt: bool,
    /// 校准计算功能使能
    ///
    /// 二进制位第3位:
    /// - 0--校准计算功能被禁用，输出的数据为ADC输出的原始数据
    /// - 1--校准计算功能被启用，输出的数据为校准后的数据
    pub calibration_enabled: bool,
    /// 表示OTP存储器数据完整性测试(CRC)结果
    ///
    /// 二进制位第4位:
    /// - 0--表示完整性测试失败，表明OTP数据存在错误
    /// - 1--表示OTP存储器数据完整性测试(CRC)通过
    pub crc_ok: bool,
    /// 工作模式
    ///
    /// 二进制位第5、6位:
    /// - 00--当前处于NORmode
    /// - 01--当前处于CYCmode
    /// - 1x--当前处于CMDmode(x表示任意值)
    pub mode: WorkingMode,
    /// 是否繁忙
    ///
    /// 二进制位第7位:
    /// - 0--传感器闲，处于休眠状态
    /// - 1--传感器忙，处于正在进行测量中
    pub is_busy: bool,
}

/// 实现AHT30温度湿度传感器状态操作
impl Status {
    /// 解析状态
    pub fn from(data: u8) -> Self {
        Self {
            _0_1: (),
            cmp_interrupt: (data & 0b00000100) != 0,
            calibration_enabled: (data & 0b00001000) != 0,
            crc_ok: (data & 0b00010000) != 0,
            mode: if (data & 0b01000000) != 0 || (data & 0b01100000) != 0 {
                WorkingMode::CMD
            } else if (data & 0b00100000) != 0 {
                WorkingMode::CYC
            } else {
                WorkingMode::NOR
            },
            is_busy: (data & 0b10000000) != 0,
        }
    }
}
