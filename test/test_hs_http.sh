#!/bin/bash

BASE_URL="http://localhost:8080"
CHANNEL_ID=1

echo "=========================================="
echo "HS-08R-16R HTTP API 快速测试"
echo "=========================================="
echo ""

# 测试1: 开启通道1
echo "【测试1】开启通道1"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"channel_on\",\"arguments\":{\"channel\":1}}" | jq
sleep 1
echo ""

# 测试2: 读取状态
echo "【测试2】读取设备状态"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"read_status\",\"arguments\":{}}" | jq
echo ""

# 测试3: 关闭通道1
echo "【测试3】关闭通道1"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"channel_off\",\"arguments\":{\"channel\":1}}" | jq
sleep 1
echo ""

# 测试4: 设置延时参数
echo "【测试4】设置通道1延时参数"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"set_delay\",\"arguments\":{\"channel\":1,\"on_delay_ms\":2000,\"off_delay_ms\":1000}}" | jq
echo ""

# 测试5: 延时开启
echo "【测试5】延时开启通道1 (2秒后生效)"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"delayed_on\",\"arguments\":{\"channel\":1}}" | jq
sleep 3
echo ""

# 测试6: 批量读取前6路
echo "【测试6】批量读取前6路通道状态"
curl -s -X POST $BASE_URL/device/readMany \
  -H 'Content-Type: application/json' \
  -d '{"global_ids":[1,2,3,4,5,6]}' | jq
echo ""

# 测试7: 一键开启
echo "【测试7】一键开启所有通道"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"all_on\",\"arguments\":{}}" | jq
sleep 1
echo ""

# 测试8: 读取最终状态
echo "【测试8】读取最终状态"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"read_status\",\"arguments\":{}}" | jq
echo ""

# 测试9: 一键关闭
echo "【测试9】一键关闭所有通道"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"all_off\",\"arguments\":{}}" | jq
echo ""

# 测试10: 读取设备地址
echo "【测试10】读取设备地址"
curl -s -X POST $BASE_URL/device/callMethod \
  -H 'Content-Type: application/json' \
  -d "{\"channel_id\":$CHANNEL_ID,\"method_name\":\"read_address\",\"arguments\":{}}" | jq
echo ""

echo "=========================================="
echo "测试完成!"
echo "=========================================="
