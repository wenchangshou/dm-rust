#!/bin/bash
# Mock 协议测试脚本

BASE_URL="http://localhost:18080"
CHANNEL_ID=1

echo "=========================================="
echo "Mock 协议测试脚本"
echo "=========================================="
echo ""

# 1. 测试 Ping
echo "1. 测试连接 (Ping)..."
curl -s -X POST "$BASE_URL/device/executeCommand" \
  -H "Content-Type: application/json" \
  -d "{\"channel_id\": $CHANNEL_ID, \"command\": \"ping\", \"params\": {}}" | jq '.'
echo ""

# 2. 写入数据
echo "2. 写入数据 (global_id=1, value=100)..."
curl -s -X POST "$BASE_URL/device/write" \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1, "value": 100}' | jq '.'
echo ""

# 3. 读取数据
echo "3. 读取数据 (global_id=1)..."
curl -s -X POST "$BASE_URL/device/read" \
  -H "Content-Type: application/json" \
  -d '{"global_id": 1}' | jq '.'
echo ""

# 4. 批量写入
echo "4. 批量写入..."
curl -s -X POST "$BASE_URL/device/writeMany" \
  -H "Content-Type: application/json" \
  -d '{"writes": [{"global_id": 1, "value": 111}, {"global_id": 2, "value": 222}]}' | jq '.'
echo ""

# 5. 批量读取
echo "5. 批量读取..."
curl -s -X POST "$BASE_URL/device/readMany" \
  -H "Content-Type: application/json" \
  -d '{"reads": [1, 2]}' | jq '.'
echo ""

# 6. 获取所有值
echo "6. 获取所有存储的值..."
curl -s -X POST "$BASE_URL/device/executeCommand" \
  -H "Content-Type: application/json" \
  -d "{\"channel_id\": $CHANNEL_ID, \"command\": \"get_all_values\", \"params\": {}}" | jq '.'
echo ""

# 7. 获取统计信息
echo "7. 获取统计信息..."
curl -s -X POST "$BASE_URL/device/callMethod" \
  -H "Content-Type: application/json" \
  -d "{\"channel_id\": $CHANNEL_ID, \"method\": \"get_statistics\", \"args\": {}}" | jq '.'
echo ""

# 8. 获取支持的方法
echo "8. 获取支持的方法列表..."
curl -s -X POST "$BASE_URL/device/getMethods" \
  -H "Content-Type: application/json" \
  -d "{\"channel_id\": $CHANNEL_ID}" | jq '.'
echo ""

# 9. 获取通道状态
echo "9. 获取通道状态..."
curl -s -X POST "$BASE_URL/device/getAllStatus" \
  -H "Content-Type: application/json" \
  -d '{}' | jq '.'
echo ""

# 10. 测试场景执行
echo "10. 执行场景 (scene_id=1)..."
curl -s -X POST "$BASE_URL/device/executeScene" \
  -H "Content-Type: application/json" \
  -d '{"scene_id": 1}' | jq '.'
echo ""

echo "=========================================="
echo "测试完成!"
echo "=========================================="
