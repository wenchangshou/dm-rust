#!/bin/bash

# HTTP API 测试脚本
# 用于测试设备控制系统的 HTTP 接口

BASE_URL="http://localhost:8080"
HEADER="Content-Type: application/json"

echo "=========================================="
echo "设备控制系统 HTTP API 测试"
echo "=========================================="
echo ""

# 测试 1: 系统健康检查
echo "1. 测试系统连接..."
curl -s $BASE_URL
echo -e "\n"

# 测试 2: 获取所有通道状态
echo "2. 获取所有通道状态..."
curl -s -X POST $BASE_URL/device/getAllStatus | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/getAllStatus
echo -e "\n"

# 测试 3: 获取所有节点状态
echo "3. 获取所有节点状态..."
curl -s -X POST $BASE_URL/device/getAllNodeStates | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/getAllNodeStates
echo -e "\n"

# 测试 4: 获取单个节点状态
echo "4. 获取节点 1 的状态..."
curl -s -X POST $BASE_URL/device/getNodeState \
  -H "$HEADER" \
  -d '{"id":1}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/getNodeState -H "$HEADER" -d '{"id":1}'
echo -e "\n"

# 测试 5: 写入设备（开启）
echo "5. 开启节点 1..."
curl -s -X POST $BASE_URL/device/write \
  -H "$HEADER" \
  -d '{"id":1,"value":1}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/write -H "$HEADER" -d '{"id":1,"value":1}'
echo -e "\n"

# 等待 2 秒
echo "等待 2 秒..."
sleep 2

# 测试 6: 读取设备
echo "6. 读取节点 1 的值..."
curl -s -X POST $BASE_URL/device/read \
  -H "$HEADER" \
  -d '{"id":1}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/read -H "$HEADER" -d '{"id":1}'
echo -e "\n"

# 测试 7: 执行场景
echo "7. 执行 '开机场景'..."
curl -s -X POST $BASE_URL/device/executeScene \
  -H "$HEADER" \
  -d '{"name":"开机场景"}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/executeScene -H "$HEADER" -d '{"name":"开机场景"}'
echo -e "\n"

# 等待 5 秒
echo "等待 5 秒..."
sleep 5

# 测试 8: 写入设备（关闭）
echo "8. 关闭节点 1..."
curl -s -X POST $BASE_URL/device/write \
  -H "$HEADER" \
  -d '{"id":1,"value":0}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/write -H "$HEADER" -d '{"id":1,"value":0}'
echo -e "\n"

# 测试 9: 执行通道命令
echo "9. 执行通道命令..."
curl -s -X POST $BASE_URL/device/executeCommand \
  -H "$HEADER" \
  -d '{"channel_id":1,"command":"get_status","params":{}}' | jq '.' 2>/dev/null || curl -s -X POST $BASE_URL/device/executeCommand -H "$HEADER" -d '{"channel_id":1,"command":"get_status","params":{}}'
echo -e "\n"

echo "=========================================="
echo "测试完成"
echo "=========================================="
