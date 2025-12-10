#!/usr/bin/env python3
"""
批量读取接口测试和演示脚本

功能:
1. 批量读取多个 Modbus 点位
2. 跨通道批量读取
3. 实时监控演示
4. 错误处理演示
"""

import requests
import time
import json
from datetime import datetime
from typing import List, Dict, Any, Optional

class DeviceClient:
    """设备控制客户端"""
    
    def __init__(self, base_url: str = "http://localhost:8080"):
        self.base_url = base_url
    
    def batch_read(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        批量读取数据
        
        Args:
            items: 读取项列表，每项包含 name, channel_id 和协议相关参数
        
        Returns:
            API 响应结果
        """
        url = f"{self.base_url}/device/batchRead"
        try:
            response = requests.post(url, json={"items": items}, timeout=10)
            response.raise_for_status()
            return response.json()
        except requests.exceptions.RequestException as e:
            return {
                "state": -1,
                "message": f"请求失败: {e}",
                "data": []
            }
    
    def batch_read_dict(self, items: List[Dict[str, Any]]) -> Dict[str, Any]:
        """
        批量读取并返回字典格式 {name: value}
        
        Args:
            items: 读取项列表
        
        Returns:
            {name: value} 字典，只包含成功读取的项
        """
        result = self.batch_read(items)
        return {
            item["name"]: item.get("value") 
            for item in result.get("data", [])
            if item.get("success", False)
        }
    
    def batch_read_with_metadata(self, items: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """
        批量读取并保留完整元数据
        
        Returns:
            包含成功/失败信息的完整列表
        """
        result = self.batch_read(items)
        return result.get("data", [])


class RealtimeMonitor:
    """实时监控"""
    
    def __init__(self, client: DeviceClient, items: List[Dict], interval: float = 1.0):
        self.client = client
        self.items = items
        self.interval = interval
        self.running = False
    
    def start(self):
        """开始实时监控"""
        self.running = True
        print("=" * 80)
        print("实时监控开始 (按 Ctrl+C 停止)")
        print("=" * 80)
        
        try:
            while self.running:
                timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                result = self.client.batch_read(self.items)
                
                print(f"\n[{timestamp}] 状态: {result['message']}")
                print("-" * 80)
                
                for item in result.get("data", []):
                    status = "✓" if item.get("success") else "✗"
                    name = item["name"]
                    
                    if item.get("success"):
                        value = item["value"]
                        print(f"  {status} {name:20s} = {value}")
                    else:
                        error = item.get("error", "未知错误")
                        print(f"  {status} {name:20s} - 错误: {error}")
                
                time.sleep(self.interval)
                
        except KeyboardInterrupt:
            print("\n\n监控停止")
            self.running = False


def demo_basic_batch_read():
    """示例1: 基础批量读取"""
    print("\n" + "=" * 80)
    print("示例 1: 基础批量读取")
    print("=" * 80)
    
    client = DeviceClient()
    
    # 定义读取项
    items = [
        {
            "name": "温度传感器",
            "channel_id": 3,
            "addr": 100,
            "type": "int16"
        },
        {
            "name": "压力传感器",
            "channel_id": 3,
            "addr": 200,
            "type": "float32"
        },
        {
            "name": "流量计",
            "channel_id": 3,
            "addr": 300,
            "type": "uint32"
        }
    ]
    
    print("\n读取配置:")
    for item in items:
        print(f"  - {item['name']}: Channel {item['channel_id']}, Addr {item['addr']}, Type {item['type']}")
    
    print("\n发送请求...")
    result = client.batch_read(items)
    
    print(f"\n响应状态: {result['state']}")
    print(f"响应消息: {result['message']}")
    print("\n读取结果:")
    
    for item in result.get("data", []):
        if item.get("success"):
            print(f"  ✓ {item['name']}: {item['value']}")
        else:
            print(f"  ✗ {item['name']}: {item.get('error', '未知错误')}")


def demo_cross_channel_read():
    """示例2: 跨通道批量读取"""
    print("\n" + "=" * 80)
    print("示例 2: 跨通道批量读取")
    print("=" * 80)
    
    client = DeviceClient()
    
    # 从多个不同通道读取数据
    items = [
        {"name": "1号机温度", "channel_id": 3, "addr": 100, "type": "int16"},
        {"name": "2号机温度", "channel_id": 4, "addr": 100, "type": "int16"},
        {"name": "3号机温度", "channel_id": 5, "addr": 100, "type": "int16"},
        {"name": "1号机压力", "channel_id": 3, "addr": 200, "type": "float32"},
        {"name": "2号机压力", "channel_id": 4, "addr": 200, "type": "float32"},
    ]
    
    print("\n跨 3 个通道读取 5 个数据点...")
    result = client.batch_read(items)
    
    print(f"\n{result['message']}")
    print("\n按通道分组显示:")
    
    # 按通道分组
    by_channel = {}
    for item in result.get("data", []):
        # 从原始配置中找到 channel_id
        channel_id = next(i["channel_id"] for i in items if i["name"] == item["name"])
        if channel_id not in by_channel:
            by_channel[channel_id] = []
        by_channel[channel_id].append(item)
    
    for channel_id, channel_items in sorted(by_channel.items()):
        print(f"\n  通道 {channel_id}:")
        for item in channel_items:
            if item.get("success"):
                print(f"    ✓ {item['name']}: {item['value']}")
            else:
                print(f"    ✗ {item['name']}: {item.get('error')}")


def demo_data_processing():
    """示例3: 数据处理与转换"""
    print("\n" + "=" * 80)
    print("示例 3: 数据处理与转换")
    print("=" * 80)
    
    client = DeviceClient()
    
    # 定义带缩放系数的读取项
    items_config = [
        {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16", "scale": 0.1, "unit": "°C"},
        {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32", "scale": 1, "unit": "Pa"},
        {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32", "scale": 0.001, "unit": "m³/h"},
        {"name": "液位", "channel_id": 3, "addr": 400, "type": "uint16", "scale": 0.01, "unit": "m"},
    ]
    
    # 提取读取项（不包含 scale 和 unit）
    read_items = [
        {k: v for k, v in item.items() if k in ["name", "channel_id", "addr", "type"]}
        for item in items_config
    ]
    
    print("\n读取原始数据...")
    result = client.batch_read(read_items)
    
    print("\n原始数据 → 工程数据:")
    print("-" * 80)
    
    for item in result.get("data", []):
        if not item.get("success"):
            continue
        
        # 找到配置
        config = next(c for c in items_config if c["name"] == item["name"])
        
        raw_value = item["value"]
        scaled_value = raw_value * config["scale"]
        
        print(f"  {config['name']:10s}: {raw_value:12} (原始) → {scaled_value:12.3f} {config['unit']}")


def demo_error_handling():
    """示例4: 错误处理"""
    print("\n" + "=" * 80)
    print("示例 4: 错误处理演示")
    print("=" * 80)
    
    client = DeviceClient()
    
    # 故意包含一些错误的配置
    items = [
        {"name": "正常点位", "channel_id": 3, "addr": 100, "type": "int16"},
        {"name": "不存在的通道", "channel_id": 999, "addr": 100, "type": "int16"},
        {"name": "错误地址", "channel_id": 3, "addr": 99999, "type": "int16"},
        {"name": "类型不匹配", "channel_id": 3, "addr": 200, "type": "invalid_type"},
    ]
    
    print("\n测试配置（含错误项）:")
    for item in items:
        print(f"  - {item['name']}")
    
    print("\n执行批量读取...")
    result = client.batch_read(items)
    
    print(f"\n{result['message']}")
    print("\n详细结果:")
    
    success_count = 0
    error_count = 0
    
    for item in result.get("data", []):
        if item.get("success"):
            success_count += 1
            print(f"  ✓ {item['name']:20s} = {item['value']}")
        else:
            error_count += 1
            print(f"  ✗ {item['name']:20s} - {item.get('error', '未知错误')}")
    
    print(f"\n统计: 成功 {success_count}, 失败 {error_count}")


def demo_realtime_monitoring():
    """示例5: 实时监控"""
    print("\n" + "=" * 80)
    print("示例 5: 实时监控演示")
    print("=" * 80)
    
    client = DeviceClient()
    
    # 定义监控点位
    items = [
        {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
        {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
        {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"},
        {"name": "泵状态", "channel_id": 3, "addr": 0, "type": "bool"},
        {"name": "阀门位置", "channel_id": 3, "addr": 400, "type": "uint16"},
    ]
    
    print("\n监控点位:")
    for item in items:
        print(f"  - {item['name']}")
    
    print("\n更新间隔: 2 秒")
    input("\n按 Enter 开始监控...")
    
    monitor = RealtimeMonitor(client, items, interval=2.0)
    monitor.start()


def demo_dict_mode():
    """示例6: 字典模式（简化访问）"""
    print("\n" + "=" * 80)
    print("示例 6: 字典模式 - 简化数据访问")
    print("=" * 80)
    
    client = DeviceClient()
    
    items = [
        {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
        {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
        {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"},
    ]
    
    print("\n使用字典模式读取...")
    data = client.batch_read_dict(items)
    
    print("\n可直接通过名称访问数据:")
    print(f"  temperature = data['温度'] / 10")
    print(f"  pressure = data['压力']")
    print(f"  flow = data['流量']")
    
    if data:
        print("\n实际值:")
        for name, value in data.items():
            print(f"  {name}: {value}")
    else:
        print("\n没有成功读取到数据")


def demo_json_export():
    """示例7: 数据导出为 JSON"""
    print("\n" + "=" * 80)
    print("示例 7: 数据导出为 JSON")
    print("=" * 80)
    
    client = DeviceClient()
    
    items = [
        {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
        {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
        {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"},
    ]
    
    print("\n读取数据...")
    result = client.batch_read(items)
    
    # 构建导出数据
    export_data = {
        "timestamp": datetime.now().isoformat(),
        "device": "设备1",
        "readings": {}
    }
    
    for item in result.get("data", []):
        if item.get("success"):
            export_data["readings"][item["name"]] = {
                "value": item["value"],
                "success": True
            }
        else:
            export_data["readings"][item["name"]] = {
                "error": item.get("error"),
                "success": False
            }
    
    # 输出 JSON
    json_str = json.dumps(export_data, ensure_ascii=False, indent=2)
    print("\n导出的 JSON 数据:")
    print(json_str)
    
    # 可以保存到文件
    # with open("data_export.json", "w", encoding="utf-8") as f:
    #     f.write(json_str)


def main():
    """主函数 - 运行所有演示"""
    print("""
╔════════════════════════════════════════════════════════════════════════════╗
║                      批量读取接口测试和演示脚本                              ║
║                                                                            ║
║  请确保:                                                                    ║
║  1. dm-rust 服务已启动 (cargo run)                                         ║
║  2. 配置文件中有对应的通道（channel_id: 3, 4, 5 等）                        ║
║  3. Modbus 设备已连接并配置正确                                             ║
╚════════════════════════════════════════════════════════════════════════════╝
    """)
    
    # 检查服务是否运行
    try:
        response = requests.get("http://localhost:8080/", timeout=2)
        print("✓ 服务连接正常\n")
    except:
        print("✗ 无法连接到服务，请先启动 dm-rust")
        print("  运行: cd dm-rust && cargo run\n")
        return
    
    demos = [
        ("基础批量读取", demo_basic_batch_read),
        ("跨通道批量读取", demo_cross_channel_read),
        ("数据处理与转换", demo_data_processing),
        ("错误处理演示", demo_error_handling),
        ("字典模式", demo_dict_mode),
        ("JSON 导出", demo_json_export),
        ("实时监控", demo_realtime_monitoring),
    ]
    
    while True:
        print("\n" + "=" * 80)
        print("选择要运行的演示:")
        print("=" * 80)
        
        for i, (name, _) in enumerate(demos, 1):
            print(f"  {i}. {name}")
        
        print(f"  {len(demos) + 1}. 运行所有演示（不含实时监控）")
        print("  0. 退出")
        
        try:
            choice = input("\n请选择 [0-{}]: ".format(len(demos) + 1)).strip()
            
            if choice == "0":
                print("\n再见！")
                break
            
            choice_num = int(choice)
            
            if choice_num == len(demos) + 1:
                # 运行所有（除了实时监控）
                for name, func in demos[:-1]:  # 排除最后一个（实时监控）
                    func()
                    time.sleep(1)
            elif 1 <= choice_num <= len(demos):
                demos[choice_num - 1][1]()
            else:
                print("无效选择")
        
        except KeyboardInterrupt:
            print("\n\n用户中断")
            break
        except ValueError:
            print("请输入数字")
        except Exception as e:
            print(f"错误: {e}")


if __name__ == "__main__":
    main()
