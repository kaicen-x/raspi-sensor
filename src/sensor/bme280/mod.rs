use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rppal::i2c::I2c;

/// BME280传感器校准参数结构体
///
/// 该结构体存储了从传感器 NVM 中读取的所有校准参数，用于
/// 温度、压力和湿度测量的精确补偿计算。这些参数在生产过程
/// 中经过精密校准，确保传感器的高精度性能。
///
/// # 存储分布
/// - 温度/压力参数: 地址 0x88-0xA1 (24字节)
/// - 湿度参数: 地址 0xA1, 0xE1-0xE7 (7字节)
///
/// # 重要性
/// 校准参数消除了传感器制造差异，提供：
/// - 温度依赖性补偿
/// - 非线性响应校正  
/// - 长期稳定性保证
/// - 交叉敏感性消除
#[derive(Debug, Default, Clone)]
struct Calibration {
    // 温度校准参数组
    /// 温度校准系数 1 - 基准偏移量
    ///
    /// ## 特性
    /// - **类型**: 无符号 16 位整数 (u16)
    /// - **地址**: 0x88-0x89 (小端序)
    /// - **范围**: 27500-28000 (典型值)
    /// - **作用**: 温度补偿计算中的基准参考值
    ///
    /// ## 计算公式
    /// ```rust
    /// var1 = (((adc_T >> 3) - (dig_T1 << 1)) * dig_T2) >> 11;
    /// ```
    pub dig_t1: u16,

    /// 温度校准系数 2 - 一阶灵敏度系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)  
    /// - **地址**: 0x8A-0x8B (小端序)
    /// - **范围**: -1000 到 1000 (典型值)
    /// - **作用**: 温度线性补偿系数，代表传感器灵敏度
    ///
    /// ## 物理意义
    /// 该参数补偿温度传感器的线性响应特性，确保在
    /// 整个工作温度范围内的测量一致性。
    pub dig_t2: i16,

    /// 温度校准系数 3 - 二阶非线性系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x8C-0x8D (小端序)
    /// - **范围**: -500 到 500 (典型值)
    /// - **作用**: 温度非线性补偿系数
    ///
    /// ## 计算公式
    /// ```rust
    /// var2 = ((((adc_T >> 4) - dig_T1) *
    ///         ((adc_T >> 4) - dig_T1)) >> 12) * dig_T3) >> 14;
    /// ```
    pub dig_t3: i16,

    // 压力校准参数组
    /// 压力校准系数 1 - 基准压力系数
    ///
    /// ## 特性
    /// - **类型**: 无符号 16 位整数 (u16)
    /// - **地址**: 0x8E-0x8F (小端序)
    /// - **范围**: 35000-38000 (典型值)
    /// - **作用**: 压力补偿的基准缩放系数
    ///
    /// ## 计算公式
    /// ```rust
    /// var1 = (((32768 + var1) * dig_P1) >> 15);
    /// ```
    pub dig_p1: u16,

    /// 压力校准系数 2 - 一阶温度补偿系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x90-0x91 (小端序)
    /// - **范围**: -10000 到 10000 (典型值)
    /// - **作用**: 压力测量的温度依赖性一阶补偿
    ///
    /// ## 物理意义
    /// 补偿压力传感器对温度变化的线性响应，确保
    /// 在不同环境温度下的压力测量稳定性。
    pub dig_p2: i16,

    /// 压力校准系数 3 - 二阶温度补偿系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x92-0x93 (小端序)
    /// - **范围**: -5000 到 5000 (典型值)
    /// - **作用**: 压力测量的温度依赖性二阶补偿
    ///
    /// ## 应用场景
    /// 用于校正压力传感器在极端温度条件下的非线性行为，
    /// 提高全温度范围内的测量精度。
    pub dig_p3: i16,

    /// 压力校准系数 4 - 压力灵敏度系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x94-0x95 (小端序)
    /// - **范围**: -20000 到 20000 (典型值)
    /// - **作用**: 压力传感器灵敏度调整系数
    ///
    /// ## 计算公式
    /// ```rust
    /// var2 = (var2 >> 2) + (dig_P4 << 16);
    /// ```
    pub dig_p4: i16,

    /// 压力校准系数 5 - 温度漂移补偿系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x96-0x97 (小端序)
    /// - **范围**: -10000 到 10000 (典型值)
    /// - **作用**: 压力测量的温度漂移补偿
    ///
    /// ## 物理意义
    /// 补偿压力传感器随温度变化的零点漂移现象，
    /// 确保长期测量稳定性。
    pub dig_p5: i16,

    /// 压力校准系数 6 - 非线性校正系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x98-0x99 (小端序)
    /// - **范围**: -5000 到 5000 (典型值)
    /// - **作用**: 压力传感器的非线性响应校正
    ///
    /// ## 计算公式
    /// ```rust
    /// var2 = (((var1 >> 2) * (var1 >> 2)) >> 11) * dig_P6;
    /// ```
    pub dig_p6: i16,

    /// 压力校准系数 7 - 零点偏移补偿系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x9A-0x9B (小端序)
    /// - **范围**: -1000 到 1000 (典型值)
    /// - **作用**: 压力测量的零点偏移补偿
    ///
    /// ## 应用场景
    /// 校正制造过程中产生的固有零点误差，确保
    /// 绝对压力测量的准确性。
    pub dig_p7: i16,

    /// 压力校准系数 8 - 高阶温度补偿系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x9C-0x9D (小端序)
    /// - **范围**: -500 到 500 (典型值)
    /// - **作用**: 压力测量的高阶温度补偿
    ///
    /// ## 物理意义
    /// 处理压力传感器在特定温度区间的复杂温度依赖性，
    /// 提供更精细的温度补偿。
    pub dig_p8: i16,

    /// 压力校准系数 9 - 最终精度调整系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0x9E-0x9F (小端序)
    /// - **范围**: -200 到 200 (典型值)
    /// - **作用**: 压力测量的最终精度微调
    ///
    /// ## 计算公式
    /// ```rust
    /// var1 = (dig_P9 * (((p >> 3) * (p >> 3)) >> 13)) >> 12;
    /// ```
    pub dig_p9: i16,

    // 湿度校准参数组
    /// 湿度校准系数 1 - 基准湿度偏移量
    ///
    /// ## 特性
    /// - **类型**: 无符号 8 位整数 (u8)
    /// - **地址**: 0xA1
    /// - **范围**: 0-100 (典型值)
    /// - **作用**: 湿度补偿的基准偏移量
    ///
    /// ## 计算公式
    /// ```rust
    /// v_x1 = v_x1 - ((((v_x1 >> 15) * (v_x1 >> 15)) >> 7) * dig_H1) >> 4;
    /// ```
    pub dig_h1: u8,

    /// 湿度校准系数 2 - 主要灵敏度系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0xE1-0xE2 (小端序)
    /// - **范围**: -2000 到 2000 (典型值)
    /// - **作用**: 湿度传感器的主要灵敏度调整
    ///
    /// ## 物理意义
    /// 确定湿度传感器的基本响应特性，是湿度补偿的
    /// 核心参数之一。
    pub dig_h2: i16,

    /// 湿度校准系数 3 - 温度交叉补偿系数
    ///
    /// ## 特性
    /// - **类型**: 无符号 8 位整数 (u8)
    /// - **地址**: 0xE3
    /// - **范围**: 0-50 (典型值)
    /// - **作用**: 湿度测量的温度依赖性补偿
    ///
    /// ## 应用场景
    /// 补偿湿度传感器读数受环境温度影响的现象，
    /// 确保在不同温度下的湿度测量一致性。
    pub dig_h3: u8,

    /// 湿度校准系数 4 - 复合温度补偿系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0xE4 (高4位) + 0xE5 (低4位)
    /// - **范围**: -1000 到 1000 (典型值)
    /// - **作用**: 湿度温度交叉补偿系数
    ///
    /// ## 存储格式
    /// 需要特殊处理：0xE4[7:4] | 0xE5[3:0]
    /// ```rust
    /// dig_h4 = (i16::from(byte_e4) << 4) | (i16::from(byte_e5) & 0x0F);
    /// ```
    pub dig_h4: i16,

    /// 湿度校准系数 5 - 非线性校正系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 16 位整数 (i16)
    /// - **地址**: 0xE5 (高4位) + 0xE6 (低4位)
    /// - **范围**: -500 到 500 (典型值)
    /// - **作用**: 湿度传感器的非线性响应校正
    ///
    /// ## 存储格式
    /// 需要特殊处理：0xE5[7:4] | 0xE6[3:0]
    /// ```rust
    /// dig_h5 = (i16::from(byte_e6) << 4) | (i16::from(byte_e5) >> 4);
    /// ```
    pub dig_h5: i16,

    /// 湿度校准系数 6 - 最终精度调整系数
    ///
    /// ## 特性
    /// - **类型**: 有符号 8 位整数 (i8)
    /// - **地址**: 0xE7
    /// - **范围**: -10 到 10 (典型值)
    /// - **作用**: 湿度测量的最终精度微调
    ///
    /// ## 物理意义
    /// 提供湿度测量的最后阶段精度调整，确保在
    /// 整个测量范围内的最佳性能。
    pub dig_h6: i8,
}

/// BME280 大气压力、温度、湿度传感器封装对象
pub struct BME280 {
    /// I2C通信句柄
    i2c_handle: Arc<Mutex<I2c>>,
    /// I2C从设备地址
    /// - BME280的地址通常为: 0x76
    i2c_addr: u8,
    /// 校准参数
    calib: Calibration,
}

/// 实现BME280传感器操作
impl BME280 {
    /// 检查传感器是否就绪
    fn check_ready(&mut self) -> anyhow::Result<()> {
        // 获取I2C总线通信权限
        let mut i2c_handle_lock = self
            .i2c_handle
            .lock()
            .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;

        // 设置从设备地址
        i2c_handle_lock.set_slave_address(self.i2c_addr as u16)?;

        // 获取状态
        let mut status = [0u8];
        i2c_handle_lock.write_read(&[0xF3], &mut status)?;

        // 检查状态
        if status[0] & 0x01 != 0 {
            return Err(anyhow::anyhow!("传感器正在更新校准数据"));
        }

        Ok(())
    }

    /// 创建BME280传感器实例
    pub fn new(i2c_handle: Arc<Mutex<I2c>>, i2c_addr: u8) -> anyhow::Result<Self> {
        // 构建传感器实例
        let mut sensor = BME280 {
            i2c_handle,
            i2c_addr,
            calib: Calibration::default(),
        };

        // 传感器上电后必须等待2ms以上
        thread::sleep(Duration::from_millis(3));

        // 检查传感器是否就绪
        sensor.check_ready()?;

        // 读取校准数据
        sensor.read_calibration_data()?;

        // 初始化传感器
        {
            // 获取I2C总线通信权限
            let mut i2c_handle_lock = sensor
                .i2c_handle
                .lock()
                .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;

            // 设置从设备地址
            i2c_handle_lock.set_slave_address(sensor.i2c_addr as u16)?;

            // 配置湿度采样率 (osrs_h = 1x)
            i2c_handle_lock.write(&[0xF2, 0x01])?;
            thread::sleep(Duration::from_millis(10));

            // 配置温度、压力采样率 (osrs_t = 1x, osrs_p = 1x) 和正常模式
            i2c_handle_lock.write(&[0xF4, 0x27])?; // 00100111 = 0x27
            thread::sleep(Duration::from_millis(10));

            // 配置滤波器关闭，待机时间 0.5ms
            i2c_handle_lock.write(&[0xF5, 0x00])?;
            thread::sleep(Duration::from_millis(10));
        }

        // OK
        Ok(sensor)
    }

    /// 读取校准数据
    fn read_calibration_data(&mut self) -> anyhow::Result<()> {
        // 获取I2C总线通信权限
        let mut i2c_handle_lock = self
            .i2c_handle
            .lock()
            .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;

        // 设置从设备地址
        i2c_handle_lock.set_slave_address(self.i2c_addr as u16)?;

        // 读取温度/压力校准参数 (0x88-0x9F)
        let mut calib_data = [0u8; 24];
        i2c_handle_lock.write_read(&[0x88], &mut calib_data)?;

        // 保存校准数据
        self.calib.dig_t1 = u16::from_le_bytes([calib_data[0], calib_data[1]]);
        self.calib.dig_t2 = i16::from_le_bytes([calib_data[2], calib_data[3]]);
        self.calib.dig_t3 = i16::from_le_bytes([calib_data[4], calib_data[5]]);
        self.calib.dig_p1 = u16::from_le_bytes([calib_data[6], calib_data[7]]);
        self.calib.dig_p2 = i16::from_le_bytes([calib_data[8], calib_data[9]]);
        self.calib.dig_p3 = i16::from_le_bytes([calib_data[10], calib_data[11]]);
        self.calib.dig_p4 = i16::from_le_bytes([calib_data[12], calib_data[13]]);
        self.calib.dig_p5 = i16::from_le_bytes([calib_data[14], calib_data[15]]);
        self.calib.dig_p6 = i16::from_le_bytes([calib_data[16], calib_data[17]]);
        self.calib.dig_p7 = i16::from_le_bytes([calib_data[18], calib_data[19]]);
        self.calib.dig_p8 = i16::from_le_bytes([calib_data[20], calib_data[21]]);
        self.calib.dig_p9 = i16::from_le_bytes([calib_data[22], calib_data[23]]);

        // 读取湿度校准参数 (0xA1, 0xE1-0xE7)
        let mut hum_calib = [0u8; 7];
        i2c_handle_lock.write_read(&[0xA1], &mut hum_calib[0..1])?;
        i2c_handle_lock.write_read(&[0xE1], &mut hum_calib[1..7])?;

        // 保存校准数据
        self.calib.dig_h1 = hum_calib[0];
        self.calib.dig_h2 = i16::from_le_bytes([hum_calib[1], hum_calib[2]]);
        self.calib.dig_h3 = hum_calib[3];
        self.calib.dig_h4 = (i16::from(hum_calib[4]) << 4) | (i16::from(hum_calib[5]) & 0x0F);
        self.calib.dig_h5 = (i16::from(hum_calib[6]) << 4) | (i16::from(hum_calib[5]) >> 4);
        self.calib.dig_h6 = hum_calib[6] as i8;

        // OK
        Ok(())
    }

    /// 验证和改进的原始数据读取函数
    fn read_raw_data(&self) -> anyhow::Result<(i32, i32, i32)> {
        // 声明缓冲区
        let mut data = [0u8; 8];

        // 确保最小作用域
        {
            // 获取I2C总线通信权限
            let mut i2c_handle_lock = self
                .i2c_handle
                .lock()
                .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;
            // 设置从设备地址
            i2c_handle_lock.set_slave_address(self.i2c_addr as u16)?;

            // 读取原始数据
            i2c_handle_lock.write_read(&[0xF7], &mut data)?;
        }

        // 解析20位压力数据 (0xF7-0xF9)
        let press_msb = data[0] as i32;
        let press_lsb = data[1] as i32;
        let press_xlsb = data[2] as i32;
        let press_raw = (press_msb << 12) | (press_lsb << 4) | (press_xlsb >> 4);

        // 解析20位温度数据 (0xFA-0xFC)
        let temp_msb = data[3] as i32;
        let temp_lsb = data[4] as i32;
        let temp_xlsb = data[5] as i32;
        let temp_raw = (temp_msb << 12) | (temp_lsb << 4) | (temp_xlsb >> 4);

        // 解析16位湿度数据 (0xFD-0xFE)
        let hum_msb = data[6] as i32;
        let hum_lsb = data[7] as i32;
        let hum_raw = (hum_msb << 8) | hum_lsb;

        // 验证数据范围
        if press_raw < 0 || press_raw > 0xFFFFF {
            return Err(anyhow::anyhow!("Pressure out of range"));
        }
        if temp_raw < 0 || temp_raw > 0xFFFFF {
            return Err(anyhow::anyhow!("Temperature out of range"));
        }
        if hum_raw < 0 || hum_raw > 0xFFFF {
            return Err(anyhow::anyhow!("Humidity out of range"));
        }

        // OK
        Ok((press_raw, temp_raw, hum_raw))
    }

    /// BME280温度补偿函数
    ///
    /// ## 功能描述
    /// 根据数据手册 4.2.3 节的温度补偿公式，将原始 ADC 温度值转换为
    /// 摄氏度温度，并生成用于压力/湿度补偿的 t_fine 值。
    ///
    /// ## 参数
    /// - `adc_t`: 从寄存器 0xFA-0xFC 读取的原始20位温度ADC值
    ///
    /// ## 返回
    /// - `(f32, i64)`: 元组包含补偿后的温度值(°C)和 t_fine 值
    ///
    /// ## 算法特点
    /// - 使用二阶多项式补偿温度传感器的非线性响应
    /// - 生成高精度中间值 t_fine 用于后续计算
    /// - 提供 0.01°C 的分辨率
    ///
    /// ## 精度指标
    /// - 分辨率: 0.01°C
    /// - 绝对精度: ±0.5°C (0-65°C范围内)
    /// - 长期稳定性: ±0.08°C/年
    fn compensate_temperature(&self, adc_t: i32) -> (f32, i64) {
        // 提取温度补偿数据编译换算（注意温度补偿运算是在32位有符号整型下转换的）
        let dig_t1 = self.calib.dig_t1 as i32;
        let dig_t2 = self.calib.dig_t2 as i32;
        let dig_t3 = self.calib.dig_t3 as i32;
        // 带入公式进行换算
        let var1 = (((adc_t >> 3) - (dig_t1 << 1)) * dig_t2) >> 11;
        let var2 = ((((adc_t >> 4) - dig_t1) * ((adc_t >> 4) - dig_t1)) >> 12) * dig_t3;
        let var2 = var2 >> 14;

        // 计算中间变量(后面的压力转换和湿度转换需要依赖温度的变化做补偿)
        let t_fine = (var1 as i64) + (var2 as i64);
        // 换算位摄氏度
        let temperature = (t_fine * 5 + 128) >> 8; // in 0.01°C

        // OK
        ((temperature as f64 / 100.0) as f32, t_fine)
    }

    /// BME280 压力补偿函数
    ///
    /// ## 功能描述
    /// 根据数据手册 4.2.3 节的压力补偿公式，将原始 ADC 压力值转换为
    /// 以帕斯卡(Pa)为单位的压力值，使用温度补偿生成的 t_fine 值。
    ///
    /// ## 参数
    /// - `adc_p`: 从寄存器 0xF7-0xF9 读取的原始20位压力ADC值
    /// - `t_fine`: 从温度补偿计算得到的高精度温度中间值
    ///
    /// ## 返回
    /// - `f32`: 补偿后的压力值(Pa)
    ///
    /// ## 算法特点
    /// - 使用复杂的多项式补偿压力传感器的非线性响应
    /// - 包含温度依赖性补偿和灵敏度校正
    /// - 提供 0.18Pa 的分辨率
    ///
    /// ## 精度指标
    /// - 分辨率: 0.18Pa (相当于1.7cm高度)
    /// - 绝对精度: ±1.0hPa (300-1100hPa, 0-65°C)
    /// - 温度系数: ±1.5Pa/K
    fn compensate_pressure(&self, adc_p: i32, t_fine: i64) -> f32 {
        // 提取压力补偿数据编译换算（注意压力补偿运算是在64位有符号整型下转换的）
        let dig_p1 = self.calib.dig_p1 as i64;
        let dig_p2 = self.calib.dig_p2 as i64;
        let dig_p3 = self.calib.dig_p3 as i64;
        let dig_p4 = self.calib.dig_p4 as i64;
        let dig_p5 = self.calib.dig_p5 as i64;
        let dig_p6 = self.calib.dig_p6 as i64;
        let dig_p7 = self.calib.dig_p7 as i64;
        let dig_p8 = self.calib.dig_p8 as i64;
        let dig_p9 = self.calib.dig_p9 as i64;

        // 步骤1: 计算温度相关变量
        // var1 = t_fine - 128000
        let mut var1 = t_fine - 128000;

        // 步骤2: 计算二阶补偿项
        // var2 = var1 * var1 * dig_P6
        let mut var2 = var1 * var1 * dig_p6;
        // var2 = var2 + (var1 * dig_P5 << 17)
        var2 = var2 + ((var1 * dig_p5) << 17);
        // var2 = var2 + (dig_P4 << 35)
        var2 = var2 + (dig_p4 << 35);

        // 步骤3: 计算主补偿项
        // var1 = ((var1 * var1 * dig_P3) >> 8) + ((var1 * dig_P2) << 12)
        var1 = ((var1 * var1 * dig_p3) >> 8) + ((var1 * dig_p2) << 12);
        // var1 = (((1 << 47) + var1) * dig_P1) >> 33
        var1 = ((((1_i64) << 47) + var1) * dig_p1) >> 33;

        // 步骤4: 检查除零错误
        // 避免因除零导致的异常
        if var1 == 0 {
            return 0.0;
        }

        // 步骤5: 计算初步压力值
        // p = 1048576 - adc_p
        let mut p = 1048576 - (adc_p as i64);
        // p = ((p << 31) - var2) * 3125 / var1
        p = (((p << 31) - var2) * 3125) / var1;

        // 步骤6: 应用最终补偿
        // var1 = (dig_P9 * (p>>13) * (p>>13)) >> 25
        var1 = (dig_p9 * ((p >> 13) * (p >> 13))) >> 25;
        // var2 = (dig_P8 * p) >> 19
        var2 = (dig_p8 * p) >> 19;
        // p = ((p + var1 + var2) >> 8) + (dig_P7 << 4)
        p = ((p + var1 + var2) >> 8) + (dig_p7 << 4);

        // 返回压力值
        (p as f64 / 256.0) as f32
    }

    /// 补偿湿度数据 - 修正版本
    ///
    /// ## 算法说明
    /// 根据数据手册 4.2.3 节的湿度补偿公式实现
    /// 使用分步计算提高可读性和可靠性
    ///
    /// ## 参数
    /// - `adc_h`: 从寄存器 0xFD-0xFE 读取的原始16位湿度ADC值
    ///
    /// ## 返回
    /// - `f32`: 补偿后的湿度值(%RH)，范围 0.0-100.0
    fn compensate_humidity(&self, adc_h: i32, t_fine: i64) -> f32 {
        // 提取湿度补偿数据编译换算（注意湿度补偿运算是在32位有符号整型下转换的）
        let dig_h1 = self.calib.dig_h1 as i32;
        let dig_h2 = self.calib.dig_h2 as i32;
        let dig_h3 = self.calib.dig_h3 as i32;
        let dig_h4 = self.calib.dig_h4 as i32;
        let dig_h5 = self.calib.dig_h5 as i32;
        let dig_h6 = self.calib.dig_h6 as i32;

        // 步骤1: 计算温度调整项
        // var1 = t_fine - 76800
        let var1 = (t_fine - 76800) as i32;

        // 步骤2: 复杂的主补偿计算
        let var2 = (((adc_h << 14) - (dig_h4 << 20) - (dig_h5 * var1)) + 16384) >> 15;
        let var3 = (((var1 * dig_h6) >> 10) * (((var1 * dig_h3) >> 11) + 32768)) >> 10;
        let var4 = ((var3 + 2097152) * dig_h2 + 8192) >> 14;
        let mut var5 = var2 * var4;

        // 步骤3: 非线性补偿
        var5 = var5 - (((((var5 >> 15) * (var5 >> 15)) >> 7) * dig_h1) >> 4);

        // 步骤4: 限制输出范围
        var5 = if var5 < 0 { 0 } else { var5 };
        var5 = if var5 > 419430400 { 419430400 } else { var5 };

        // 返回相对湿度: Q22.10格式的湿度值 / 1024
        (((var5 >> 12) as u32) / 1024) as f32
    }

    /// 读取补偿后的传感器数据
    ///
    /// - 返回（温度【℃】，空气压力【Pa】，湿度【%RH】）
    pub fn read(&mut self) -> anyhow::Result<(f32, f32, f32)> {
        // 读取原始数据
        let (adc_p, adc_t, adc_h) = self.read_raw_data()?;

        // 使用补偿公式补偿数据
        let (temperature, t_fine) = self.compensate_temperature(adc_t);
        let pressure = self.compensate_pressure(adc_p, t_fine);
        let humidity = self.compensate_humidity(adc_h, t_fine);

        // OK
        Ok((temperature, pressure, humidity))
    }

    /// 软复位传感器
    pub fn reset(&mut self) -> anyhow::Result<()> {
        // 最小化锁作用域
        {
            // 获取I2C总线通信权限
            let mut i2c_handle_lock = self
                .i2c_handle
                .lock()
                .map_err(|err| anyhow::anyhow!("I2C通信总线繁忙: {}", err))?;

            // 设置从设备地址
            i2c_handle_lock.set_slave_address(self.i2c_addr as u16)?;

            // 软重置
            i2c_handle_lock.write(&[0xE0, 0xB6])?;
        }

        // 等待重置完成
        thread::sleep(Duration::from_millis(5));

        // 重新读取校准数据
        self.read_calibration_data()
    }
}
