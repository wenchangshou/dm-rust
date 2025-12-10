#!/bin/bash

# 数据库 API 测试脚本 - 仅添加操作（不删除）
# 测试 Screen、Material、Resource 的创建和查询

BASE_URL="${API_URL:-http://localhost:18080}"
HEADER="Content-Type: application/json"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 计数器
TOTAL=0
PASSED=0
FAILED=0

# 保存创建的 ID
SCREEN_ID=""
RESOURCE_ID=""
MATERIAL_ID=""

echo "=========================================="
echo "数据库 API 测试 (仅添加，不删除)"
echo "=========================================="
echo "服务地址: $BASE_URL"
echo ""

# 检查服务是否可用
echo "检查服务是否可用..."
if ! curl -s "$BASE_URL/api/screens" > /dev/null 2>&1; then
    echo -e "${RED}✗ 服务未启动，请先启动服务${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 服务已启动${NC}"
echo ""

# ==================== Screen API ====================
echo "========== Screen API 测试 =========="
echo ""

# 1. 获取所有 Screen
echo -e "${BLUE}[1] 获取所有 Screen${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/screens")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
count=$(echo "$response" | jq -r '.data | length' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (现有 $count 条记录)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 2. 创建 Screen
echo -e "${BLUE}[2] 创建 Screen${NC}"
TOTAL=$((TOTAL + 1))
TIMESTAMP=$(date +%s)
response=$(curl -s -X POST "$BASE_URL/api/screens" \
    -H "$HEADER" \
    -d "{
        \"type\": \"Normal\",
        \"name\": \"测试屏幕_$TIMESTAMP\",
        \"content\": \"{\\\"title\\\": \\\"测试标题\\\", \\\"background\\\": \\\"#ffffff\\\"}\",
        \"active\": true
    }")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
SCREEN_ID=$(echo "$response" | jq -r '.data.id' 2>/dev/null)
if [ "$state" == "0" ] && [ -n "$SCREEN_ID" ] && [ "$SCREEN_ID" != "null" ]; then
    echo -e "${GREEN}✓ 通过${NC} (创建 Screen ID: $SCREEN_ID)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 3. 获取单个 Screen
echo -e "${BLUE}[3] 获取刚创建的 Screen${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/screens/$SCREEN_ID")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 4. 按类型查询 Screen
echo -e "${BLUE}[4] 按类型查询 Screen (Normal)${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/screens?type=Normal")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
count=$(echo "$response" | jq -r '.data | length' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (找到 $count 条记录)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# ==================== Resource API ====================
echo "========== Resource API 测试 =========="
echo ""

# 5. 获取所有 Resource
echo -e "${BLUE}[5] 获取所有 Resource${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/resources")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
count=$(echo "$response" | jq -r '.data | length' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (现有 $count 条记录)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 6. 上传测试文件
echo -e "${BLUE}[6] 上传测试文件${NC}"
TOTAL=$((TOTAL + 1))
TEST_FILE="/tmp/test_upload_$$.txt"
echo "这是测试文件内容 - $(date)" > "$TEST_FILE"

response=$(curl -s -X POST "$BASE_URL/api/resources/upload" \
    -F "file=@$TEST_FILE" \
    -F "name=测试资源_$TIMESTAMP" \
    -F "resource_type=document")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
RESOURCE_ID=$(echo "$response" | jq -r '.data.resource.id' 2>/dev/null)
RESOURCE_PATH=$(echo "$response" | jq -r '.data.resource.path' 2>/dev/null)
RESOURCE_URL=$(echo "$response" | jq -r '.data.url' 2>/dev/null)
if [ "$state" == "0" ] && [ -n "$RESOURCE_ID" ] && [ "$RESOURCE_ID" != "null" ]; then
    echo -e "${GREEN}✓ 通过${NC} (创建 Resource ID: $RESOURCE_ID)"
    echo "  文件路径: $RESOURCE_PATH"
    echo "  访问 URL: $RESOURCE_URL"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
rm -f "$TEST_FILE"
echo ""

# 7. 获取单个 Resource
echo -e "${BLUE}[7] 获取刚创建的 Resource${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/resources/$RESOURCE_ID")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 8. 按类型查询 Resource
echo -e "${BLUE}[8] 按类型查询 Resource (document)${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/resources?resource_type=document")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
count=$(echo "$response" | jq -r '.data | length' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (找到 $count 条记录)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 9. 访问静态文件
echo -e "${BLUE}[9] 访问上传的静态文件${NC}"
TOTAL=$((TOTAL + 1))
http_code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL$RESOURCE_URL")
if [ "$http_code" == "200" ]; then
    echo -e "${GREEN}✓ 通过${NC} (HTTP $http_code)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC} (HTTP $http_code)"
    FAILED=$((FAILED + 1))
fi
echo "  URL: $BASE_URL$RESOURCE_URL"
echo ""

# ==================== Material API ====================
echo "========== Material API 测试 =========="
echo ""

# 10. 获取所有 Material
echo -e "${BLUE}[10] 获取所有 Material${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/materials")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
count=$(echo "$response" | jq -r '.data | length' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (现有 $count 条记录)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 11. 创建 Material（关联 Resource）
echo -e "${BLUE}[11] 创建 Material（关联 Resource）${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X POST "$BASE_URL/api/materials" \
    -H "$HEADER" \
    -d "{
        \"name\": \"测试素材_$TIMESTAMP\",
        \"resource_id\": \"$RESOURCE_ID\"
    }")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
MATERIAL_ID=$(echo "$response" | jq -r '.data.id' 2>/dev/null)
MATERIAL_PATH=$(echo "$response" | jq -r '.data.path' 2>/dev/null)
if [ "$state" == "0" ] && [ -n "$MATERIAL_ID" ] && [ "$MATERIAL_ID" != "null" ]; then
    echo -e "${GREEN}✓ 通过${NC} (创建 Material ID: $MATERIAL_ID)"
    echo "  返回的 path: $MATERIAL_PATH"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 12. 获取单个 Material（验证返回 path）
echo -e "${BLUE}[12] 获取单个 Material（验证返回 path）${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/materials/$MATERIAL_ID")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
returned_path=$(echo "$response" | jq -r '.data.path' 2>/dev/null)
if [ "$state" == "0" ] && [ -n "$returned_path" ] && [ "$returned_path" != "null" ] && [ "$returned_path" != "" ]; then
    echo -e "${GREEN}✓ 通过${NC} (返回 path: $returned_path)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC} (path 为空或 null)"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 13. 按名称查询 Material
echo -e "${BLUE}[13] 按名称查询 Material${NC}"
TOTAL=$((TOTAL + 1))
SEARCH_NAME=$(echo -n '测试' | jq -sRr @uri)
response=$(curl -s -X GET "$BASE_URL/api/materials?name=$SEARCH_NAME")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
count=$(echo "$response" | jq -r '.data | length' 2>/dev/null)
if [ "$state" == "0" ] && [ "$count" -gt "0" ] 2>/dev/null; then
    echo -e "${GREEN}✓ 通过${NC} (找到 $count 个匹配结果)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC} (state=$state, count=$count)"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# ==================== 汇总 ====================
echo "=========================================="
echo "测试结果汇总"
echo "=========================================="
echo "总测试数: $TOTAL"
echo -e "通过: ${GREEN}$PASSED${NC}"
echo -e "失败: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
else
    echo -e "${RED}✗ 有 $FAILED 个测试失败${NC}"
fi

echo ""
echo "=========================================="
echo "创建的数据 ID（可用于后续操作）"
echo "=========================================="
echo "Screen ID:   $SCREEN_ID"
echo "Resource ID: $RESOURCE_ID"
echo "Material ID: $MATERIAL_ID"
echo ""
echo -e "${YELLOW}注意: 本脚本不删除创建的数据，可手动删除或用于后续测试${NC}"
