#!/bin/bash
# HS-08R-16R 电源时序器测试脚本

BASE_URL="http://localhost:8080"
CHANNEL_ID=1

echo "=========================================="
echo "HS-08R-16R 多功能电源时序器 测试"
echo "=========================================="

# 测试1: 通道1开
echo -e "\n【测试1】打开通道1:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"channel_on\",
    \"args\": {\"channel\": 1}
  }" | jq .

sleep 1

# 测试2: 读取设备状态
echo -e "\n【测试2】读取设备状态:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"read_status\",
    \"args\": {}
  }" | jq .

sleep 1

# 测试3: 通道1关
echo -e "\n【测试3】关闭通道1:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"channel_off\",
    \"args\": {\"channel\": 1}
  }" | jq .

sleep 1

# 测试4: 设置通道1开延时为1秒
echo -e "\n【测试4】设置通道1开延时为1秒 (1000ms):"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"set_delay\",
    \"args\": {
      \"channel\": 1,
      \"delay_ms\": 1000,
      \"is_on\": true
    }
  }" | jq .

sleep 1

# 测试5: 设置通道2开延时为2秒
echo -e "\n【测试5】设置通道2开延时为2秒 (2000ms):"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"set_delay\",
    \"args\": {
      \"channel\": 2,
      \"delay_ms\": 2000,
      \"is_on\": true
    }
  }" | jq .

sleep 1

# 测试6: 延时开 (按设定延时依次开启)
echo -e "\n【测试6】执行延时开 (通道将按延时参数依次开启):"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"delayed_on\",
    \"args\": {}
  }" | jq .

echo -e "\n等待3秒,让延时开启完成..."
sleep 3

# 测试7: 读取状态确认
echo -e "\n【测试7】延时开启后读取状态:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"read_status\",
    \"args\": {}
  }" | jq .

sleep 1

# 测试8: 一键关闭所有通道
echo -e "\n【测试8】一键关闭所有通道:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"all_off\",
    \"args\": {}
  }" | jq .

sleep 1

# 测试9: 一键开启所有通道
echo -e "\n【测试9】一键开启所有通道:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"all_on\",
    \"args\": {}
  }" | jq .

sleep 1

# 测试10: 读取设备地址
echo -e "\n【测试10】读取设备地址:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"read_address\",
    \"args\": {}
  }" | jq .

sleep 1

# 测试11: 设置时间 (2025年11月11日 15:30:00)
echo -e "\n【测试11】设置设备时间为 2025-11-11 15:30:00:"
curl -s -X POST "${BASE_URL}/device/customMethod" \
  -H "Content-Type: application/json" \
  -d "{
    \"channel_id\": ${CHANNEL_ID},
    \"method\": \"set_time\",
    \"args\": {
      \"year\": 25,
      \"month\": 11,
      \"day\": 11,
      \"hour\": 15,
      \"minute\": 30,
      \"second\": 0
    }
  }" | jq .

sleep 1

# 测试12: 使用标准节点接口 - 写入
echo -e "\n【测试12】使用标准写接口开启通道3 (global_id=3):"
curl -s -X POST "${BASE_URL}/device/write" \
  -H "Content-Type: application/json" \
  -d '{"global_id": 3, "value": 1}' | jq .

sleep 1

# 测试13: 使用标准节点接口 - 读取
echo -e "\n【测试13】使用标准读接口读取通道3状态:"
curl -s -X GET "${BASE_URL}/device/read?global_id=3" | jq .

sleep 1

# 测试14: 批量写入
echo -e "\n【测试14】批量开启通道1,2,3:"
curl -s -X POST "${BASE_URL}/device/writeMany" \
  -H "Content-Type: application/json" \
  -d '{
    "items": [
      {"id": 1, "value": 1},
      {"id": 2, "value": 1},
      {"id": 3, "value": 1}
    ]
  }' | jq .

sleep 1

# 测试15: 批量读取
echo -e "\n【测试15】批量读取通道1-5状态:"
curl -s -X POST "${BASE_URL}/device/readMany" \
  -H "Content-Type: application/json" \
  -d '{"ids": [1, 2, 3, 4, 5]}' | jq .

echo -e "\n=========================================="
echo "测试完成"
echo "=========================================="
echo ""
echo "提示:"
echo "- 通道编号: 1-12"
echo "- 延时单位: 毫秒 (ms)"
echo "- 设备地址出厂默认为 1"
echo "- RS485只能写,读取需要RS232"
echo ""
