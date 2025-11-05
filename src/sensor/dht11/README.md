## DHT11传感器

### 注意
1. 该示例以焊接好上拉电阻的DHT11模块为实验对象,引脚为3针,分别是VCC(3.3v),DATA,GND.
2. 接线前需要先启用树莓派的One-Wire接口协议,可通过一下方式启用
    - 方式一: 通过`raspi-config`工具启用
    ```shell
    // 使用树莓派官方工具
    sudo raspi-config
    // 按一下选项操作
    // `Interface Options` > `1-Wire` > `Yes` > `OK` > `Finish`
    // 操作完成后重启即可
    sudo reboot
    ```

    - 方式二:在系统中修改`/boot/config.txt`配置，新的镜像已改为`/boot/firmware/config.txt`,在文件末尾添加如下内容：
    ```txt
    dtparam=i2c_arm=on
    dtparam=spi=on
    dtoverlay=dht11
    ```
    添加完成后`sudo reboot`重启即可


    - 方式三:通过SD卡读卡器修改`config.txt`,在文件末尾添加如下内容：
    ```txt
    dtparam=i2c_arm=on
    dtparam=spi=on
    dtoverlay=dht11
    ```
3. 将VCC针脚接入到树莓派的3.3v引脚，GND接到树莓派的GND引脚，DATA针脚接到树莓派的GPIO4（物理引脚7）引脚上，使用该类即可读取温度和湿度。
4. 受DHT11的专有协议以及One-Wire协议的约束，会有读取失败的情况，多读取几次取正常的就行。


### 时序图
参考网友的连接，[https://blog.csdn.net/weixin_40394827/article/details/117912454](https://blog.csdn.net/weixin_40394827/article/details/117912454)