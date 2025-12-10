#!/bin/bash
# 测试批量读取接口

BASE_URL="http://localhost:8080"

echo "=========================================="
echo "测试批量读取接口 (readMany)"
echo "=========================================="

# 测试1: 读取所有三个节点
echo -e "\n【测试1】读取所有节点 (ids: [1, 2, 3]):"
curl -X POST "${BASE_URL}/device/readMany" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": [1, 2, 3]
  }' | jq .

# 测试2: 只读取节点1和节点2
echo -e "\n【测试2】只读取节点1和2 (ids: [1, 2]):"
curl -X POST "${BASE_URL}/device/readMany" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": [1, 2]
  }' | jq .

# 测试3: 读取单个节点
echo -e "\n【测试3】读取单个节点 (id: 2, addr: 21):"
curl -X POST "${BASE_URL}/device/readMany" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": [2]
  }' | jq .

# 测试4: 读取不存在的节点 (应该返回错误)
echo -e "\n【测试4】读取不存在的节点 (id: 999):"
curl -X POST "${BASE_URL}/device/readMany" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": [1, 999, 3]
  }' | jq .

# 测试5: 空列表
echo -e "\n【测试5】空ID列表 (ids: []):"
curl -X POST "${BASE_URL}/device/readMany" \
  -H "Content-Type: application/json" \
  -d '{
    "ids": []
  }' | jq .

echo -e "\n=========================================="
echo "测试完成"
echo "=========================================="
echo ""
echo "说明:"
echo "- 节点1 (global_id=1, addr=0): scale=0.1"
echo "- 节点2 (global_id=2, addr=21): scale=0.01"
echo "- 节点3 (global_id=3, addr=30): float32 无scale"
echo ""
echo "如果 addr=21 的值是 200, 应该返回 2.0 (200 * 0.01)"
echo ""
