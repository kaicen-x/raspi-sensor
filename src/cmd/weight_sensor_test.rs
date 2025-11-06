use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicI32, Ordering},
    },
};
use std::{thread, time::Duration};

use raspi_sensor::sensor::button::Button;
use raspi_sensor::sensor::hx711::{Gain, HX711};

// Button接入GPIO针脚
const BUTTON_PIN: u8 = 17;
// HX711传感器接入GPIO针脚
const HX711_DATA_PIN: u8 = 23;
const HX711_CLOCK_PIN: u8 = 24;

fn main() -> anyhow::Result<()> {
    // 创建按钮实例
    let mut button = Button::new(BUTTON_PIN)?;
    // 创建HX711称重传感器实例
    let mut hx711 = HX711::new(HX711_CLOCK_PIN, HX711_DATA_PIN, Gain::ChannelA128)?;
    // 缓存的皮重(克)
    let tare_weight: Arc<AtomicI32> = Arc::new(AtomicI32::new(0));
    // 缓存的当前重量（克）
    let current_weight: Arc<AtomicI32> = Arc::new(AtomicI32::new(0));
    // 三次缓存（克），用来判断是否稳定
    let mut weight_cache: VecDeque<i32> = VecDeque::with_capacity(3);

    // 读取一次有效重量,最多尝试10次（以实现开机去皮）
    for _ in 0..10 {
        // 尝试读取当前重量
        {
            if let Ok(data) = hx711.read() {
                println!("开机去皮重量读取成功");
                // 转换为克
                let weight = data * 2 / 1000;
                // 设置皮重
                tare_weight.store(weight, Ordering::Release);
                // 储存当前重量
                current_weight.store(weight, Ordering::Release);
                // 跳出
                println!("开机去皮配置成功");
                break;
            } else {
                println!("开机去皮读取重量失败");
            }
        }
        // 等待100ms, 不然HX711芯片处理不过来
        std::thread::sleep(Duration::from_millis(100));
    }

    // 克隆一个皮重引用
    let tare_weight_clone = tare_weight.clone();
    // 克隆当前重量引用
    let current_weight_clone = current_weight.clone();
    // 监听按钮状态变化
    button.on_change(move |state| {
        // 当按钮按下时执行去皮
        if state {
            println!("去皮按钮已按下");
            // 将当前重量储存为新的皮重
            let new_weight = current_weight_clone.load(Ordering::Acquire);
            println!("新皮重: {}", new_weight);
            tare_weight_clone.store(new_weight, Ordering::Release);
        }
    })?;

    // 循环显示重量
    loop {
        // 读取数据,传感器输出重量单位为KG，移除三位小数
        match hx711.read() {
            Ok(data) => {
                // 转换为克
                let mut weight = data * 2 / 1000;
                // 储存当前重量
                current_weight.store(weight, Ordering::Release);
                // 获取已配置的皮重
                let tare_weight_tmp = tare_weight.load(Ordering::Acquire);
                // 去除皮重
                weight = weight - tare_weight_tmp;
                // 校准重量
                weight = ((weight as f32) * 1.4) as i32;
                
                // 执行滤波，防止微小变化
                // 和上一次的缓存相比，正负1g以内认为无变化，仍然用上一次的重量
                if let Some(last_weight) = weight_cache.back() {
                    if (weight - last_weight).abs() <= 1 {
                        weight = *last_weight;
                    }
                }

                // 缓存满了时需要移除首位
                if weight_cache.len() == 3 {
                    weight_cache.pop_front();
                }

                // 缓存重量
                weight_cache.push_back(weight);

                // 是否稳定
                let stabel = if weight_cache.len() == 3 {
                    (weight_cache[0] - weight_cache[1]).abs() <= 1
                        && (weight_cache[0] - weight_cache[2]).abs() <= 1
                } else {
                    false
                };
                // 输出当前重量
                println!("读取重量成功: {}g, 稳定: {}", weight, stabel);
            }
            Err(err) => {
                eprintln!("读取重量失败: {}", err);
            }
        }

        // 等待100ms, 不然HX711芯片处理不过来
        thread::sleep(Duration::from_millis(100));
    }
}
