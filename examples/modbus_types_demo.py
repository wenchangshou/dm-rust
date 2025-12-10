#!/usr/bin/env python3
"""
Modbus 数据类型使用示例

演示如何使用不同的数据类型读写 Modbus 设备
"""

import requests
import json
import time
from typing import Any, Optional

BASE_URL = "http://localhost:8080"


class ModbusClient:
    """Modbus 客户端封装"""
    
    def __init__(self, base_url: str = BASE_URL):
        self.base_url = base_url
    
    def read_typed(self, channel: int, addr: int, data_type: str) -> Optional[Any]:
        """
        读取指定类型的数据
        
        Args:
            channel: 通道号
            addr: 寄存器地址
            data_type: 数据类型（uint16, int16, uint32, int32, float32, float64, bool等）
        
        Returns:
            读取的值，失败返回 None
        """
        try:
            response = requests.post(
                f"{self.base_url}/device/execute",
                json={
                    "channel": channel,
                    "command": "read",
                    "params": {
                        "addr": addr,
                        "type": data_type
                    }
                },
                timeout=5
            )
            
            if response.status_code == 200:
                result = response.json()
                if result["code"] == 0:
                    return result["data"]["value"]
                else:
                    print(f"错误: {result['msg']}")
            else:
                print(f"HTTP错误: {response.status_code}")
        except Exception as e:
            print(f"读取异常: {e}")
        
        return None
    
    def write_typed(self, channel: int, addr: int, value: Any, data_type: str) -> bool:
        """
        写入指定类型的数据
        
        Args:
            channel: 通道号
            addr: 寄存器地址
            value: 要写入的值
            data_type: 数据类型
        
        Returns:
            成功返回 True，失败返回 False
        """
        try:
            response = requests.post(
                f"{self.base_url}/device/execute",
                json={
                    "channel": channel,
                    "command": "write",
                    "params": {
                        "addr": addr,
                        "type": data_type,
                        "value": value
                    }
                },
                timeout=5
            )
            
            if response.status_code == 200:
                result = response.json()
                if result["code"] == 0:
                    return True
                else:
                    print(f"错误: {result['msg']}")
            else:
                print(f"HTTP错误: {response.status_code}")
        except Exception as e:
            print(f"写入异常: {e}")
        
        return False


def example_temperature_sensor():
    """示例1: 温度传感器（Int16，精度0.1°C）"""
    print("\n" + "="*60)
    print("示例1: 温度传感器（Int16，精度0.1°C）")
    print("="*60)
    
    client = ModbusClient()
    channel = 1
    addr = 100
    
    # 写入温度值 25.6°C (存储为 256)
    temp_celsius = 25.6
    temp_raw = int(temp_celsius * 10)  # 转换为整数
    
    print(f"写入温度: {temp_celsius}°C (原始值: {temp_raw})")
    if client.write_typed(channel, addr, temp_raw, "int16"):
        print("✓ 写入成功")
        
        # 读取并转换
        raw_value = client.read_typed(channel, addr, "int16")
        if raw_value is not None:
            actual_temp = raw_value / 10.0
            print(f"✓ 读取成功: {actual_temp}°C (原始值: {raw_value})")
    else:
        print("✗ 写入失败")


def example_pressure_sensor():
    """示例2: 压力传感器（Float32）"""
    print("\n" + "="*60)
    print("示例2: 压力传感器（Float32）")
    print("="*60)
    
    client = ModbusClient()
    channel = 1
    addr = 200
    
    # 写入压力值
    pressure = 101325.5  # Pa (标准大气压)
    
    print(f"写入压力: {pressure} Pa")
    if client.write_typed(channel, addr, pressure, "float32"):
        print("✓ 写入成功")
        
        # 读取
        value = client.read_typed(channel, addr, "float32")
        if value is not None:
            print(f"✓ 读取成功: {value} Pa")
            print(f"  精度损失: {abs(value - pressure)} Pa")
    else:
        print("✗ 写入失败")


def example_flow_counter():
    """示例3: 流量累计器（UInt32）"""
    print("\n" + "="*60)
    print("示例3: 流量累计器（UInt32）")
    print("="*60)
    
    client = ModbusClient()
    channel = 1
    addr = 300
    
    # 初始化计数器
    initial_count = 1234567
    
    print(f"初始化计数器: {initial_count} L")
    if client.write_typed(channel, addr, initial_count, "uint32"):
        print("✓ 写入成功")
        
        # 模拟增加
        time.sleep(0.1)
        new_count = initial_count + 100
        
        print(f"更新计数器: {new_count} L (+100)")
        if client.write_typed(channel, addr, new_count, "uint32"):
            print("✓ 更新成功")
            
            # 读取
            value = client.read_typed(channel, addr, "uint32")
            if value is not None:
                print(f"✓ 读取成功: {value} L")
                print(f"  增量: {value - initial_count} L")
    else:
        print("✗ 初始化失败")


def example_position_encoder():
    """示例4: 位置编码器（Int32，支持正负值）"""
    print("\n" + "="*60)
    print("示例4: 位置编码器（Int32，单位：μm）")
    print("="*60)
    
    client = ModbusClient()
    channel = 1
    addr = 500
    
    # 测试正负位置
    positions = [0, 1000000, -500000, 2500000]
    
    for pos in positions:
        print(f"\n设置位置: {pos} μm ({pos/1000:.1f} mm)")
        if client.write_typed(channel, addr, pos, "int32"):
            print("  ✓ 写入成功")
            
            # 读取验证
            value = client.read_typed(channel, addr, "int32")
            if value is not None:
                print(f"  ✓ 读取成功: {value} μm ({value/1000:.1f} mm)")
                if value == pos:
                    print("  ✓ 值匹配")
                else:
                    print(f"  ✗ 值不匹配（差异: {value - pos}）")
        else:
            print("  ✗ 写入失败")


def example_high_precision_scale():
    """示例5: 高精度天平（Float64）"""
    print("\n" + "="*60)
    print("示例5: 高精度天平（Float64）")
    print("="*60)
    
    client = ModbusClient()
    channel = 1
    addr = 600
    
    # 写入高精度重量
    weight = 123.456789012345  # 克
    
    print(f"写入重量: {weight:.12f} g")
    if client.write_typed(channel, addr, weight, "float64"):
        print("✓ 写入成功")
        
        # 读取
        value = client.read_typed(channel, addr, "float64")
        if value is not None:
            print(f"✓ 读取成功: {value:.12f} g")
            print(f"  精度损失: {abs(value - weight):.15e} g")
    else:
        print("✗ 写入失败")


def example_bool_controls():
    """示例6: 布尔控制（线圈）"""
    print("\n" + "="*60)
    print("示例6: 布尔控制（开关、指示灯）")
    print("="*60)
    
    client = ModbusClient()
    channel = 1
    
    # 定义控制点
    controls = {
        "启动开关": 0,
        "报警指示": 1,
        "运行指示灯": 2
    }
    
    for name, addr in controls.items():
        print(f"\n{name} (地址:{addr}):")
        
        # 设置为 True
        print("  设置为 ON (true)")
        if client.write_typed(channel, addr, True, "bool"):
            print("    ✓ 写入成功")
            
            # 读取验证
            value = client.read_typed(channel, addr, "bool")
            if value is not None:
                print(f"    ✓ 读取成功: {'ON' if value else 'OFF'}")
        
        time.sleep(0.1)
        
        # 设置为 False
        print("  设置为 OFF (false)")
        if client.write_typed(channel, addr, False, "bool"):
            print("    ✓ 写入成功")
            
            # 读取验证
            value = client.read_typed(channel, addr, "bool")
            if value is not None:
                print(f"    ✓ 读取成功: {'ON' if value else 'OFF'}")


def example_little_endian():
    """示例7: 小端序数据（UInt32LE, Float32LE）"""
    print("\n" + "="*60)
    print("示例7: 小端序数据（PLC设备）")
    print("="*60)
    
    client = ModbusClient()
    channel = 2  # 假设通道2是小端序的PLC
    
    # UInt32LE 测试
    print("\nUInt32LE 测试:")
    addr_uint32le = 100
    value_uint32le = 0x12345678
    
    print(f"  写入: 0x{value_uint32le:08X} ({value_uint32le})")
    if client.write_typed(channel, addr_uint32le, value_uint32le, "uint32le"):
        print("  ✓ 写入成功")
        
        # 读取
        result = client.read_typed(channel, addr_uint32le, "uint32le")
        if result is not None:
            print(f"  ✓ 读取成功: 0x{result:08X} ({result})")
    
    # Float32LE 测试
    print("\nFloat32LE 测试:")
    addr_float32le = 200
    value_float32le = 3.14159
    
    print(f"  写入: {value_float32le}")
    if client.write_typed(channel, addr_float32le, value_float32le, "float32le"):
        print("  ✓ 写入成功")
        
        # 读取
        result = client.read_typed(channel, addr_float32le, "float32le")
        if result is not None:
            print(f"  ✓ 读取成功: {result}")


def example_batch_monitoring():
    """示例8: 批量监控多个传感器"""
    print("\n" + "="*60)
    print("示例8: 批量监控传感器数据")
    print("="*60)
    
    client = ModbusClient()
    channel = 1
    
    # 定义传感器配置
    sensors = [
        {"name": "温度1", "addr": 100, "type": "int16", "scale": 0.1, "unit": "°C"},
        {"name": "温度2", "addr": 110, "type": "float32", "scale": 1, "unit": "°C"},
        {"name": "压力", "addr": 200, "type": "float32", "scale": 1, "unit": "Pa"},
        {"name": "流量", "addr": 300, "type": "uint32", "scale": 1, "unit": "L"},
        {"name": "转速", "addr": 400, "type": "uint16", "scale": 1, "unit": "RPM"},
        {"name": "位置", "addr": 500, "type": "int32", "scale": 0.001, "unit": "mm"},
    ]
    
    print("\n开始监控（按 Ctrl+C 停止）...\n")
    
    try:
        for i in range(3):  # 只演示3次
            print(f"--- 第 {i+1} 次读取 ---")
            
            for sensor in sensors:
                value = client.read_typed(channel, sensor["addr"], sensor["type"])
                if value is not None:
                    scaled_value = value * sensor["scale"]
                    print(f"  {sensor['name']:8s}: {scaled_value:12.2f} {sensor['unit']}")
                else:
                    print(f"  {sensor['name']:8s}: 读取失败")
            
            print()
            time.sleep(1)
    except KeyboardInterrupt:
        print("\n监控已停止")


def main():
    """主程序"""
    print("""
╔══════════════════════════════════════════════════════════╗
║     Modbus 数据类型使用示例程序                          ║
║                                                          ║
║  演示如何使用不同的数据类型读写 Modbus 设备              ║
╚══════════════════════════════════════════════════════════╝
    """)
    
    # 检查服务器连接
    try:
        response = requests.get(f"{BASE_URL}/health", timeout=2)
        if response.status_code != 200:
            print("⚠ 警告: 服务器可能未正常运行")
    except:
        print("✗ 错误: 无法连接到服务器")
        print(f"  请确保服务器正在运行: {BASE_URL}")
        return
    
    print("✓ 服务器连接正常\n")
    
    # 运行示例
    try:
        example_temperature_sensor()
        example_pressure_sensor()
        example_flow_counter()
        example_position_encoder()
        example_high_precision_scale()
        example_bool_controls()
        # example_little_endian()  # 需要实际的小端序设备
        # example_batch_monitoring()  # 需要实际设备
        
        print("\n" + "="*60)
        print("所有示例执行完成！")
        print("="*60)
        
    except Exception as e:
        print(f"\n✗ 执行异常: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
