#!/bin/bash

# Modbus 数据类型测试脚本
# 使用方法: ./test_modbus_types.sh [channel_id]

BASE_URL="http://localhost:8080"
CHANNEL=${1:-1}

echo "========================================="
echo "  Modbus 数据类型功能测试"
echo "  通道: $CHANNEL"
echo "  服务器: $BASE_URL"
echo "========================================="

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试函数
test_type() {
    local name=$1
    local addr=$2
    local type=$3
    local write_value=$4
    
    echo -e "\n${YELLOW}>>> 测试 $name ($type)${NC}"
    
    # 写入
    echo "  [写入] 地址:$addr, 值:$write_value"
    write_result=$(curl -s -X POST "$BASE_URL/device/execute" \
        -H "Content-Type: application/json" \
        -d "{\"channel\":$CHANNEL,\"command\":\"write\",\"params\":{\"addr\":$addr,\"type\":\"$type\",\"value\":$write_value}}")
    
    write_code=$(echo "$write_result" | jq -r '.code')
    if [ "$write_code" == "0" ]; then
        echo -e "  ${GREEN}✓ 写入成功${NC}"
    else
        echo -e "  ${RED}✗ 写入失败: $(echo "$write_result" | jq -r '.msg')${NC}"
        return 1
    fi
    
    # 读取
    echo "  [读取] 地址:$addr"
    read_result=$(curl -s -X POST "$BASE_URL/device/execute" \
        -H "Content-Type: application/json" \
        -d "{\"channel\":$CHANNEL,\"command\":\"read\",\"params\":{\"addr\":$addr,\"type\":\"$type\"}}")
    
    read_code=$(echo "$read_result" | jq -r '.code')
    if [ "$read_code" == "0" ]; then
        read_value=$(echo "$read_result" | jq -r '.data.value')
        registers=$(echo "$read_result" | jq -r '.data.registers // empty')
        echo -e "  ${GREEN}✓ 读取成功: $read_value${NC}"
        if [ -n "$registers" ]; then
            echo "    寄存器: $registers"
        fi
    else
        echo -e "  ${RED}✗ 读取失败: $(echo "$read_result" | jq -r '.msg')${NC}"
        return 1
    fi
}

# 检查服务器连接
echo -e "\n检查服务器连接..."
if ! curl -s -f "$BASE_URL/health" > /dev/null 2>&1; then
    echo -e "${RED}错误: 无法连接到服务器 $BASE_URL${NC}"
    echo "请确保服务器正在运行"
    exit 1
fi
echo -e "${GREEN}✓ 服务器连接正常${NC}"

# 开始测试
echo -e "\n========================================="
echo "  开始测试各种数据类型"
echo "========================================="

# 1. UInt16 测试
test_type "无符号16位整数" 100 "uint16" 12345

# 2. Int16 测试（负数）
test_type "有符号16位整数（负数）" 110 "int16" -12345

# 3. Int16 测试（正数）
test_type "有符号16位整数（正数）" 111 "int16" 5678

# 4. UInt32 测试
test_type "无符号32位整数" 200 "uint32" 987654321

# 5. Int32 测试
test_type "有符号32位整数" 210 "int32" -123456789

# 6. UInt32LE 测试（小端序）
test_type "无符号32位整数（小端）" 220 "uint32le" 111222333

# 7. Int32LE 测试（小端序）
test_type "有符号32位整数（小端）" 230 "int32le" -999888777

# 8. Float32 测试
test_type "32位浮点数" 300 "float32" 123.456

# 9. Float32 测试（负数）
test_type "32位浮点数（负数）" 310 "float32" -98.765

# 10. Float32LE 测试（小端序）
test_type "32位浮点数（小端）" 320 "float32le" 456.789

# 11. Float64 测试
test_type "64位浮点数" 400 "float64" 3.141592653589793

# 12. Float64 测试（科学计数法）
test_type "64位浮点数（大数）" 410 "float64" 1234567890.123456

# 13. Bool 测试 (true)
test_type "布尔值（true）" 5 "bool" true

# 14. Bool 测试 (false)
test_type "布尔值（false）" 6 "bool" false

# 总结
echo -e "\n========================================="
echo -e "${GREEN}  测试完成！${NC}"
echo "========================================="
echo ""
echo "提示："
echo "  - 如果测试失败，请检查 Modbus 设备是否正常运行"
echo "  - 确保地址范围在设备支持范围内"
echo "  - 某些设备可能不支持所有数据类型"
echo ""
