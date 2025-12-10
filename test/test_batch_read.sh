#!/bin/bash

# 批量读取接口测试脚本
# 使用 curl 命令测试批量读取功能

BASE_URL="http://localhost:8080"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "======================================================================"
echo "               批量读取接口测试脚本"
echo "======================================================================"
echo ""

# 检查服务是否运行
echo "检查服务连接..."
if ! curl -s "$BASE_URL/" > /dev/null 2>&1; then
    echo -e "${RED}✗ 无法连接到服务${NC}"
    echo "  请先启动 dm-rust: cd dm-rust && cargo run"
    exit 1
fi
echo -e "${GREEN}✓ 服务连接正常${NC}"
echo ""

# 测试1: 基础批量读取
echo "======================================================================"
echo "测试 1: 基础批量读取（3个 Modbus 点位）"
echo "======================================================================"
echo ""
echo "请求配置:"
echo "  - 温度传感器: Channel 3, Addr 100, Type int16"
echo "  - 压力传感器: Channel 3, Addr 200, Type float32"
echo "  - 流量计:     Channel 3, Addr 300, Type uint32"
echo ""

curl -X POST "$BASE_URL/device/batchRead" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
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
  }' 2>/dev/null | python3 -m json.tool

echo ""
echo ""
read -p "按 Enter 继续下一个测试..."
echo ""

# 测试2: 跨通道批量读取
echo "======================================================================"
echo "测试 2: 跨通道批量读取"
echo "======================================================================"
echo ""
echo "请求配置:"
echo "  - 1号机温度: Channel 3, Addr 100"
echo "  - 2号机温度: Channel 4, Addr 100"
echo "  - 3号机温度: Channel 5, Addr 100"
echo ""

curl -X POST "$BASE_URL/device/batchRead" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"name": "1号机温度", "channel_id": 3, "addr": 100, "type": "int16"},
      {"name": "2号机温度", "channel_id": 4, "addr": 100, "type": "int16"},
      {"name": "3号机温度", "channel_id": 5, "addr": 100, "type": "int16"}
    ]
  }' 2>/dev/null | python3 -m json.tool

echo ""
echo ""
read -p "按 Enter 继续下一个测试..."
echo ""

# 测试3: 混合数据类型
echo "======================================================================"
echo "测试 3: 混合数据类型读取"
echo "======================================================================"
echo ""
echo "请求配置:"
echo "  - 温度 (int16):   0.1°C 精度"
echo "  - 压力 (float32): 浮点数"
echo "  - 流量 (uint32):  整数累计"
echo "  - 液位 (uint16):  整数"
echo "  - 泵状态 (bool):  开关"
echo ""

curl -X POST "$BASE_URL/device/batchRead" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
      {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
      {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"},
      {"name": "液位", "channel_id": 3, "addr": 400, "type": "uint16"},
      {"name": "泵状态", "channel_id": 3, "addr": 0, "type": "bool"}
    ]
  }' 2>/dev/null | python3 -m json.tool

echo ""
echo ""
read -p "按 Enter 继续下一个测试..."
echo ""

# 测试4: 大批量读取
echo "======================================================================"
echo "测试 4: 大批量读取（10个点位）"
echo "======================================================================"
echo ""

curl -X POST "$BASE_URL/device/batchRead" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"name": "点位1", "channel_id": 3, "addr": 100, "type": "int16"},
      {"name": "点位2", "channel_id": 3, "addr": 101, "type": "int16"},
      {"name": "点位3", "channel_id": 3, "addr": 102, "type": "int16"},
      {"name": "点位4", "channel_id": 3, "addr": 103, "type": "int16"},
      {"name": "点位5", "channel_id": 3, "addr": 104, "type": "int16"},
      {"name": "点位6", "channel_id": 3, "addr": 200, "type": "float32"},
      {"name": "点位7", "channel_id": 3, "addr": 202, "type": "float32"},
      {"name": "点位8", "channel_id": 3, "addr": 300, "type": "uint32"},
      {"name": "点位9", "channel_id": 3, "addr": 302, "type": "uint32"},
      {"name": "点位10", "channel_id": 3, "addr": 400, "type": "uint16"}
    ]
  }' 2>/dev/null | python3 -m json.tool

echo ""
echo ""
read -p "按 Enter 继续下一个测试..."
echo ""

# 测试5: 错误处理测试
echo "======================================================================"
echo "测试 5: 错误处理（含错误配置）"
echo "======================================================================"
echo ""
echo "请求配置（包含故意的错误）:"
echo "  - 正常点位"
echo "  - 不存在的通道 (channel_id: 999)"
echo "  - 错误的地址 (addr: 99999)"
echo ""

curl -X POST "$BASE_URL/device/batchRead" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"name": "正常点位", "channel_id": 3, "addr": 100, "type": "int16"},
      {"name": "不存在的通道", "channel_id": 999, "addr": 100, "type": "int16"},
      {"name": "错误地址", "channel_id": 3, "addr": 99999, "type": "int16"}
    ]
  }' 2>/dev/null | python3 -m json.tool

echo ""
echo ""
read -p "按 Enter 继续下一个测试..."
echo ""

# 测试6: 实时监控模拟（循环读取）
echo "======================================================================"
echo "测试 6: 实时监控模拟（每2秒读取一次，共5次）"
echo "======================================================================"
echo ""

for i in {1..5}; do
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')] 第 $i 次读取${NC}"
    echo "----------------------------------------------------------------------"
    
    curl -s -X POST "$BASE_URL/device/batchRead" \
      -H "Content-Type: application/json" \
      -d '{
        "items": [
          {"name": "温度", "channel_id": 3, "addr": 100, "type": "int16"},
          {"name": "压力", "channel_id": 3, "addr": 200, "type": "float32"},
          {"name": "流量", "channel_id": 3, "addr": 300, "type": "uint32"}
        ]
      }' | python3 -c "
import sys, json
result = json.load(sys.stdin)
print(f\"  状态: {result['message']}\")
for item in result.get('data', []):
    status = '✓' if item.get('success') else '✗'
    name = item['name']
    if item.get('success'):
        print(f\"  {status} {name:10s} = {item['value']}\")
    else:
        print(f\"  {status} {name:10s} - 错误: {item.get('error', '未知')}\")
"
    
    echo ""
    
    if [ $i -lt 5 ]; then
        sleep 2
    fi
done

echo ""
echo "======================================================================"
echo "                          测试完成"
echo "======================================================================"
echo ""
echo "总结:"
echo "  ✓ 基础批量读取"
echo "  ✓ 跨通道读取"
echo "  ✓ 混合数据类型"
echo "  ✓ 大批量读取"
echo "  ✓ 错误处理"
echo "  ✓ 实时监控模拟"
echo ""
echo "查看完整文档: dm-rust/BATCH_READ_API.md"
echo ""
