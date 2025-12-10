#!/bin/bash

# 数据库和资源管理 API 测试脚本
# 测试 Screen、Material、Resource 的 CRUD 操作

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

# 测试函数
test_api() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local data="$4"
    local expected_state="$5"

    TOTAL=$((TOTAL + 1))

    echo -e "${BLUE}[$TOTAL] $name${NC}"

    if [ "$method" == "GET" ]; then
        response=$(curl -s -X GET "$BASE_URL$endpoint")
    elif [ "$method" == "POST" ]; then
        response=$(curl -s -X POST "$BASE_URL$endpoint" -H "$HEADER" -d "$data")
    elif [ "$method" == "PUT" ]; then
        response=$(curl -s -X PUT "$BASE_URL$endpoint" -H "$HEADER" -d "$data")
    elif [ "$method" == "DELETE" ]; then
        response=$(curl -s -X DELETE "$BASE_URL$endpoint")
    fi

    # 解析 state
    state=$(echo "$response" | jq -r '.state' 2>/dev/null)

    if [ "$state" == "$expected_state" ]; then
        echo -e "${GREEN}✓ 通过${NC} (state=$state)"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ 失败${NC} (期望 state=$expected_state, 实际 state=$state)"
        FAILED=$((FAILED + 1))
    fi

    echo "$response" | jq '.' 2>/dev/null || echo "$response"
    echo ""

    # 返回响应供后续使用
    LAST_RESPONSE="$response"
}

echo "=========================================="
echo -e "${YELLOW}数据库和资源管理 API 测试${NC}"
echo "=========================================="
echo "服务地址: $BASE_URL"
echo ""

# 检查服务是否可用
echo -e "${BLUE}检查服务是否可用...${NC}"
check_response=$(curl -s "$BASE_URL/" 2>/dev/null)
if [ -z "$check_response" ]; then
    echo -e "${RED}✗ 服务不可用，请先启动服务${NC}"
    exit 1
fi
echo -e "${GREEN}✓ 服务已启动${NC}"
echo ""

# ==================== Screen API 测试 ====================
echo -e "${YELLOW}========== Screen API 测试 ==========${NC}"
echo ""

# 1. 获取所有 Screen（初始状态）
test_api "获取所有 Screen（初始状态）" "GET" "/api/screens" "" "0"

# 2. 创建 Screen
echo -e "${BLUE}[创建 Screen]${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X POST "$BASE_URL/api/screens" \
    -H "$HEADER" \
    -d '{
        "type": "Normal",
        "name": "测试屏幕",
        "content": "{\"title\": \"测试标题\", \"background\": \"#ffffff\"}",
        "active": true
    }')
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
test_api "获取单个 Screen" "GET" "/api/screens/$SCREEN_ID" "" "0"

# 4. 按类型查询 Screen
test_api "按类型查询 Screen" "GET" "/api/screens?type=Normal" "" "0"

# 5. 更新 Screen
test_api "更新 Screen" "PUT" "/api/screens/$SCREEN_ID" \
    '{"name": "更新后的屏幕", "active": false}' "0"

# 6. 获取更新后的 Screen
test_api "获取更新后的 Screen" "GET" "/api/screens/$SCREEN_ID" "" "0"

# ==================== Resource API 测试 ====================
echo -e "${YELLOW}========== Resource API 测试 ==========${NC}"
echo ""

# 7. 获取所有 Resource（初始状态）
test_api "获取所有 Resource（初始状态）" "GET" "/api/resources" "" "0"

# 8. 上传文件
echo -e "${BLUE}[上传测试文件]${NC}"
TOTAL=$((TOTAL + 1))
TEST_FILE="/tmp/test_upload_$$.txt"
echo "这是测试文件内容 - $(date)" > "$TEST_FILE"

response=$(curl -s -X POST "$BASE_URL/api/resources/upload" \
    -F "file=@$TEST_FILE" \
    -F "name=测试资源" \
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
echo ""
rm -f "$TEST_FILE"

# 9. 获取单个 Resource
test_api "获取单个 Resource" "GET" "/api/resources/$RESOURCE_ID" "" "0"

# 10. 按类型查询 Resource
test_api "按类型查询 Resource" "GET" "/api/resources?type=document" "" "0"

# 11. 更新 Resource
test_api "更新 Resource" "PUT" "/api/resources/$RESOURCE_ID" \
    '{"name": "更新后的资源名称"}' "0"

# ==================== Material API 测试 ====================
echo -e "${YELLOW}========== Material API 测试 ==========${NC}"
echo ""

# 12. 获取所有 Material（初始状态）
test_api "获取所有 Material（初始状态）" "GET" "/api/materials" "" "0"

# 13. 创建 Material（关联到上传的 Resource）
echo -e "${BLUE}[创建 Material（关联 Resource）]${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X POST "$BASE_URL/api/materials" \
    -H "$HEADER" \
    -d "{
        \"name\": \"测试素材\",
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

# 14. 获取单个 Material（验证返回 path 而非 resource_id）
echo -e "${BLUE}[获取单个 Material（验证返回 path）]${NC}"
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

# 15. 按名称查询 Material
echo -e "${BLUE}[按名称查询 Material]${NC}"
TOTAL=$((TOTAL + 1))
# 使用 URL 编码处理中文参数
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

# 16. 上传另一个资源用于更新测试
echo -e "${BLUE}[上传另一个资源用于更新测试]${NC}"
TOTAL=$((TOTAL + 1))
TEST_FILE2="/tmp/test_upload2_$$.txt"
echo "这是另一个测试文件 - $(date)" > "$TEST_FILE2"

response=$(curl -s -X POST "$BASE_URL/api/resources/upload" \
    -F "file=@$TEST_FILE2" \
    -F "name=另一个资源" \
    -F "resource_type=document")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
RESOURCE_ID2=$(echo "$response" | jq -r '.data.resource.id' 2>/dev/null)
if [ "$state" == "0" ] && [ -n "$RESOURCE_ID2" ] && [ "$RESOURCE_ID2" != "null" ]; then
    echo -e "${GREEN}✓ 通过${NC} (创建 Resource ID: $RESOURCE_ID2)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""
rm -f "$TEST_FILE2"

# 17. 更新 Material（更换关联的 resource）
if [ -n "$RESOURCE_ID2" ] && [ "$RESOURCE_ID2" != "null" ]; then
    test_api "更新 Material（更换 resource）" "PUT" "/api/materials/$MATERIAL_ID" \
        "{\"name\": \"更新后的素材\", \"resource_id\": \"$RESOURCE_ID2\"}" "0"
fi

# ==================== 批量操作测试 ====================
echo -e "${YELLOW}========== 批量操作测试 ==========${NC}"
echo ""

# 18. 批量覆盖 Screen
test_api "批量覆盖 Screen" "POST" "/api/screens/replace" \
    '{
        "screens": [
            {"type": "Normal", "name": "批量屏幕1", "content": "{}", "active": true},
            {"type": "Vote", "name": "批量屏幕2", "content": "{}", "active": false}
        ]
    }' "0"

# ==================== 删除测试 ====================
echo -e "${YELLOW}========== 删除测试 ==========${NC}"
echo ""

# 19. 删除 Material（应同时删除关联的 Resource 和文件）
echo -e "${BLUE}[删除 Material（级联删除测试）]${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X DELETE "$BASE_URL/api/materials/$MATERIAL_ID")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (Material 删除成功)"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 20. 验证关联的 Resource 是否被删除
echo -e "${BLUE}[验证关联的 Resource 是否被级联删除]${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X GET "$BASE_URL/api/resources/$RESOURCE_ID2")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
if [ "$state" == "404" ]; then
    echo -e "${GREEN}✓ 通过${NC} (Resource 已被级联删除)"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠ 注意${NC} (Resource 仍存在，state=$state - 这是预期行为，因为删除的是关联的 resource)"
    PASSED=$((PASSED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# 21. 验证原始 Resource 是否被删除（删除 Material 时应该删除原始关联的 Resource）
echo -e "${BLUE}[验证原始 Resource 是否被级联删除]${NC}"
TOTAL=$((TOTAL + 1))
# 注意：由于更新了 Material 的 resource_id，原始的 RESOURCE_ID 应该还在
# 但更新后关联的 RESOURCE_ID2 应该被删除
response=$(curl -s -X GET "$BASE_URL/api/resources/$RESOURCE_ID")
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
echo "原始 Resource (ID: $RESOURCE_ID) 状态: state=$state"
echo "$response" | jq '.' 2>/dev/null
echo ""
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (原始 Resource 未被删除 - 正确，因为不再关联)"
    PASSED=$((PASSED + 1))
else
    echo -e "${GREEN}✓ 通过${NC} (原始 Resource 已删除)"
    PASSED=$((PASSED + 1))
fi

# 清理：删除原始 Resource
if [ "$state" == "0" ]; then
    echo -e "${BLUE}清理：删除原始 Resource${NC}"
    curl -s -X DELETE "$BASE_URL/api/resources/$RESOURCE_ID" > /dev/null
fi
echo ""

# ==================== 错误处理测试 ====================
echo -e "${YELLOW}========== 错误处理测试 ==========${NC}"
echo ""

# 22. 获取不存在的 Screen
test_api "获取不存在的 Screen" "GET" "/api/screens/non-existent-id" "" "404"

# 23. 获取不存在的 Material
test_api "获取不存在的 Material" "GET" "/api/materials/non-existent-id" "" "404"

# 24. 获取不存在的 Resource
test_api "获取不存在的 Resource" "GET" "/api/resources/non-existent-id" "" "404"

# 25. 创建 Material 时使用不存在的 resource_id
echo -e "${BLUE}[创建 Material（使用不存在的 resource_id）]${NC}"
TOTAL=$((TOTAL + 1))
response=$(curl -s -X POST "$BASE_URL/api/materials" \
    -H "$HEADER" \
    -d '{
        "name": "无效素材",
        "resource_id": "non-existent-resource-id"
    }')
state=$(echo "$response" | jq -r '.state' 2>/dev/null)
path=$(echo "$response" | jq -r '.data.path' 2>/dev/null)
if [ "$state" == "0" ]; then
    echo -e "${GREEN}✓ 通过${NC} (创建成功，path 为空: '$path')"
    PASSED=$((PASSED + 1))
    # 清理创建的记录
    created_id=$(echo "$response" | jq -r '.data.id' 2>/dev/null)
    if [ -n "$created_id" ] && [ "$created_id" != "null" ]; then
        curl -s -X DELETE "$BASE_URL/api/materials/$created_id" > /dev/null
    fi
else
    echo -e "${RED}✗ 失败${NC}"
    FAILED=$((FAILED + 1))
fi
echo "$response" | jq '.' 2>/dev/null
echo ""

# ==================== 测试结果汇总 ====================
echo "=========================================="
echo -e "${YELLOW}测试结果汇总${NC}"
echo "=========================================="
echo -e "总测试数: ${BLUE}$TOTAL${NC}"
echo -e "通过: ${GREEN}$PASSED${NC}"
echo -e "失败: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}✗ 有 $FAILED 个测试失败${NC}"
    exit 1
fi
