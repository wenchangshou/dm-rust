#!/bin/bash

# 自定义方法测试脚本
# 用于测试通道自定义方法调用功能

BASE_URL="http://localhost:8080"
HEADER="Content-Type: application/json"

echo "=========================================="
echo "自定义方法测试"
echo "=========================================="
echo ""

# 测试 1: 获取通道 1 支持的方法列表
echo "1. 获取通道 1 的方法列表..."
curl -s -X POST $BASE_URL/device/getMethods \
  -H "$HEADER" \
  -d '{"channel_id":1}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/getMethods -H "$HEADER" -d '{"channel_id":1}'
echo -e "\n"

# 测试 2: 调用单参数方法
echo "2. 调用 set_input 方法（切换到 HDMI1）..."
curl -s -X POST $BASE_URL/device/callMethod \
  -H "$HEADER" \
  -d '{
    "channel_id": 1,
    "method_name": "set_input",
    "arguments": {
      "source": "hdmi1"
    }
  }' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/callMethod -H "$HEADER" -d '{"channel_id":1,"method_name":"set_input","arguments":{"source":"hdmi1"}}'
echo -e "\n"

# 等待 1 秒
sleep 1

# 测试 3: 调用无参数方法
echo "3. 调用 get_lamp_hours 方法（获取灯泡时长）..."
curl -s -X POST $BASE_URL/device/callMethod \
  -H "$HEADER" \
  -d '{
    "channel_id": 1,
    "method_name": "get_lamp_hours",
    "arguments": {}
  }' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/callMethod -H "$HEADER" -d '{"channel_id":1,"method_name":"get_lamp_hours","arguments":{}}'
echo -e "\n"

# 测试 4: 调用多参数方法
echo "4. 调用 set_brightness 方法（设置亮度到 80%）..."
curl -s -X POST $BASE_URL/device/callMethod \
  -H "$HEADER" \
  -d '{
    "channel_id": 1,
    "method_name": "set_brightness",
    "arguments": {
      "level": 80,
      "transition": true
    }
  }' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/callMethod -H "$HEADER" -d '{"channel_id":1,"method_name":"set_brightness","arguments":{"level":80,"transition":true}}'
echo -e "\n"

# 测试 5: 测试不存在的方法
echo "5. 测试调用不存在的方法（预期失败）..."
curl -s -X POST $BASE_URL/device/callMethod \
  -H "$HEADER" \
  -d '{
    "channel_id": 1,
    "method_name": "nonexistent_method",
    "arguments": {}
  }' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/callMethod -H "$HEADER" -d '{"channel_id":1,"method_name":"nonexistent_method","arguments":{}}'
echo -e "\n"

# 测试 6: 测试不存在的通道
echo "6. 测试不存在的通道（预期失败）..."
curl -s -X POST $BASE_URL/device/getMethods \
  -H "$HEADER" \
  -d '{"channel_id":999}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/getMethods -H "$HEADER" -d '{"channel_id":999}'
echo -e "\n"

# 测试 7: 获取自定义协议通道的方法（如果启用）
echo "7. 获取通道 7 的方法列表（自定义协议）..."
curl -s -X POST $BASE_URL/device/getMethods \
  -H "$HEADER" \
  -d '{"channel_id":7}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/getMethods -H "$HEADER" -d '{"channel_id":7}'
echo -e "\n"

# 测试 8: 调用自定义协议的方法
echo "8. 调用自定义协议的方法..."
curl -s -X POST $BASE_URL/device/callMethod \
  -H "$HEADER" \
  -d '{
    "channel_id": 7,
    "method_name": "query_status",
    "arguments": {
      "device_id": 1,
      "detail": true
    }
  }' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/callMethod -H "$HEADER" -d '{"channel_id":7,"method_name":"query_status","arguments":{"device_id":1,"detail":true}}'
echo -e "\n"

# 测试 9: 复杂参数方法调用
echo "9. 调用复杂参数方法..."
curl -s -X POST $BASE_URL/device/callMethod \
  -H "$HEADER" \
  -d '{
    "channel_id": 7,
    "method_name": "batch_control",
    "arguments": {
      "devices": [1, 2, 3, 4],
      "action": "power_on",
      "params": {
        "delay": 1000,
        "retry": 3
      }
    }
  }' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/callMethod -H "$HEADER" -d '{"channel_id":7,"method_name":"batch_control","arguments":{"devices":[1,2,3,4],"action":"power_on","params":{"delay":1000,"retry":3}}}'
echo -e "\n"

echo "=========================================="
echo "测试完成"
echo "=========================================="
echo ""
echo "注意事项："
echo "1. 如果看到 '不支持自定义方法' 的错误，说明协议尚未实现该方法"
echo "2. 通道 7（自定义协议）默认是禁用的，需要在配置中启用才能测试"
echo "3. 实际方法实现需要在协议代码中完成，配置只是定义接口"
