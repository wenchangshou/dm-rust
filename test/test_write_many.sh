#!/bin/bash

# writeMany API 测试脚本
# 用于批量写入多个节点数据

echo "==== writeMany API 测试 ===="
echo ""

# 测试1: 写入3个节点
echo "测试1: 批量写入3个节点"
curl -X POST http://localhost:8080/device/writeMany \
  -H 'Content-Type: application/json' \
  -d '{
    "items": [
      {"id": 1, "value": 100},
      {"id": 2, "value": 200},
      {"id": 3, "value": 300}
    ]
  }'
echo -e "\n"

# 测试2: 验证写入结果 - 读取节点1
echo "验证节点1的值:"
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{"id": 1}'
echo -e "\n"

# 测试3: 验证写入结果 - 读取节点2
echo "验证节点2的值:"
curl -X POST http://localhost:8080/device/read \
  -H 'Content-Type: application/json' \
  -d '{"id": 2}'
echo -e "\n"

# 测试4: 写入包含部分失败的批次
echo "测试2: 批量写入(包含不存在的节点)"
curl -X POST http://localhost:8080/device/writeMany \
  -H 'Content-Type: application/json' \
  -d '{
    "items": [
      {"id": 1, "value": 500},
      {"id": 999, "value": 777},
      {"id": 2, "value": 600}
    ]
  }'
echo -e "\n"

echo "==== 测试完成 ===="
