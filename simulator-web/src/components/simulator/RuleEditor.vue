<template>
    <div class="rule-editor-container">
        <!-- 校验和配置 (顶部单独区域) -->
        <el-card class="checksum-card" shadow="never">
            <template #header>
                <div class="card-title">
                    <el-checkbox v-model="hasChecksum" @change="handleChecksumToggle" :disabled="readonly">启用校验和
                        (Checksum)</el-checkbox>
                </div>
            </template>
            <div v-if="config.checksum" class="checksum-config">
                <el-form :inline="true" size="small" :disabled="readonly">
                    <el-form-item label="算法">
                        <el-select v-model="config.checksum.algorithm" style="width: 140px">
                            <el-option label="CRC16 Modbus" :value="ChecksumAlgorithm.Crc16Modbus" />
                            <el-option label="Sum8" :value="ChecksumAlgorithm.Sum8" />
                            <el-option label="XOR" :value="ChecksumAlgorithm.XOR" />
                        </el-select>
                    </el-form-item>
                    <el-form-item label="起始偏移">
                        <el-input-number v-model="config.checksum.range_start" :min="0" controls-position="right"
                            style="width: 80px" />
                    </el-form-item>
                    <el-form-item label="结束偏移(倒数)">
                        <el-input-number v-model="config.checksum.range_end_offset" :min="0" controls-position="right"
                            style="width: 80px" />
                    </el-form-item>
                    <el-form-item label="大端序">
                        <el-switch v-model="config.checksum.big_endian" />
                    </el-form-item>
                </el-form>
            </div>
        </el-card>

        <div class="main-editor">
            <!-- 左侧列表 -->
            <div class="rule-list-panel">
                <div class="list-header">
                    <span>规则列表 ({{ config.rules.length }})</span>
                    <el-button v-if="!readonly" type="primary" size="small" :icon="Plus" circle @click="addRule" />
                </div>
                <div class="list-content custom-scrollbar">
                    <el-empty v-if="config.rules.length === 0" description="暂无规则" image-size="60" />
                    <div v-else v-for="(rule, index) in config.rules" :key="index"
                        :class="['rule-item', { active: selectedIndex === index }]" @click="selectRule(index)">
                        <div class="rule-item-header">
                            <span class="rule-index">#{{ index + 1 }}</span>
                            <span class="rule-name" :title="rule.name">{{ rule.name }}</span>
                        </div>
                        <div class="rule-badges">
                            <el-tag size="small" type="info" effect="plain">{{
                                getMatchPatternLabel(rule.match_pattern.type)
                                }}</el-tag>
                            <el-icon class="arrow-icon">
                                <Right />
                            </el-icon>
                            <el-tag size="small" type="success" effect="plain">{{
                                getResponseActionLabel(rule.action.type) }}</el-tag>
                        </div>
                        <div class="rule-actions" v-if="!readonly && selectedIndex === index">
                            <el-button-group size="small">
                                <el-button link size="small" :icon="Top" @click.stop="moveRule(index, -1)"
                                    :disabled="index === 0" />
                                <el-button link size="small" :icon="Bottom" @click.stop="moveRule(index, 1)"
                                    :disabled="index === config.rules.length - 1" />
                                <el-button link size="small" type="danger" :icon="Delete"
                                    @click.stop="removeRule(index)" />
                            </el-button-group>
                        </div>
                    </div>
                </div>
            </div>

            <!-- 右侧详情 -->
            <div class="rule-detail-panel">
                <div v-if="selectedIndex !== -1 && config.rules[selectedIndex]" class="detail-content custom-scrollbar">
                    <div class="detail-header">
                        <el-input v-model="config.rules[selectedIndex].name" placeholder="规则名称" :readonly="readonly"
                            class="rule-name-input">
                            <template #prefix>规则名称</template>
                        </el-input>
                    </div>

                    <el-divider content-position="left">匹配条件 (Request)</el-divider>

                    <el-form label-position="top" :disabled="readonly">
                        <el-form-item label="匹配类型">
                            <el-radio-group v-model="config.rules[selectedIndex].match_pattern.type" size="small">
                                <el-radio-button label="Hex">Hex</el-radio-button>
                                <el-radio-button label="Regex">正则(Hex)</el-radio-button>
                                <el-radio-button label="StringContain">字符串包含</el-radio-button>
                                <el-radio-button label="Any">任意(Any)</el-radio-button>
                            </el-radio-group>
                        </el-form-item>
                        <el-form-item v-if="config.rules[selectedIndex].match_pattern.type !== 'Any'" label="匹配值">
                            <el-input v-model="config.rules[selectedIndex].match_pattern.value"
                                :placeholder="getMatchPlaceholder(config.rules[selectedIndex].match_pattern.type)"
                                type="textarea" :rows="3" />
                            <div class="help-text"
                                v-if="getMatchExample(config.rules[selectedIndex].match_pattern.type)">
                                <el-icon>
                                    <InfoFilled />
                                </el-icon>
                                {{ getMatchExample(config.rules[selectedIndex].match_pattern.type) }}
                            </div>
                        </el-form-item>
                    </el-form>

                    <el-divider content-position="left">响应动作 (Response)</el-divider>

                    <el-form label-position="top" :disabled="readonly">
                        <el-form-item label="动作类型">
                            <el-select v-model="config.rules[selectedIndex].action.type" style="width: 100%">
                                <el-option-group label="静态响应">
                                    <el-option label="静态 Hex 数据" value="Static" />
                                    <el-option label="静态 String 数据" value="StaticString" />
                                </el-option-group>
                                <el-option-group label="动态响应">
                                    <el-option label="模板 (Hex 结果)" value="Template" />
                                    <el-option label="模板 (String 结果)" value="TemplateString" />
                                </el-option-group>
                                <el-option-group label="其他">
                                    <el-option label="延迟 (Delay)" value="Delay" />
                                    <el-option label="无响应 (None)" value="None" />
                                </el-option-group>
                            </el-select>
                        </el-form-item>

                        <el-form-item
                            v-if="['Static', 'StaticString', 'Template', 'TemplateString'].includes(config.rules[selectedIndex].action.type)"
                            label="响应数据">
                            <el-input v-model="config.rules[selectedIndex].action.value"
                                :placeholder="getActionPlaceholder(config.rules[selectedIndex].action.type)"
                                type="textarea" :rows="5" font-family="monospace" />
                            <div class="help-text" v-if="getActionExample(config.rules[selectedIndex].action.type)">
                                <el-icon>
                                    <InfoFilled />
                                </el-icon>
                                {{ getActionExample(config.rules[selectedIndex].action.type) }}
                            </div>
                            <div v-if="['Template', 'TemplateString'].includes(config.rules[selectedIndex].action.type)"
                                class="help-text warning">
                                <el-icon>
                                    <Warning />
                                </el-icon>
                                支持变量: &#123;&#123; key &#125;&#125;
                            </div>
                        </el-form-item>

                        <el-form-item v-if="config.rules[selectedIndex].action.type === 'Delay'" label="延迟时间 (ms)">
                            <el-input-number v-model="config.rules[selectedIndex].action.value" :min="0" :step="100"
                                style="width: 100%" />
                        </el-form-item>
                    </el-form>

                </div>
                <div v-else class="empty-selection">
                    <el-empty description="请选择左侧规则进行编辑" />
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, reactive } from 'vue'
import { Plus, Delete, Top, Bottom, Right, InfoFilled, Warning } from '@element-plus/icons-vue'
import { type CustomProtocolConfig, ChecksumAlgorithm } from '@/types/simulator'

const props = defineProps<{
    modelValue: CustomProtocolConfig
    readonly?: boolean
}>()

const emit = defineEmits<{
    'update:modelValue': [value: CustomProtocolConfig]
}>()

// 本地响应式数据
const config = reactive<CustomProtocolConfig>(JSON.parse(JSON.stringify(props.modelValue)))
const hasChecksum = ref(!!config.checksum)
const selectedIndex = ref<number>(-1)

// 初始化选中第一个规则
if (config.rules.length > 0) {
    selectedIndex.value = 0
}

// 监听 props 变化
watch(() => props.modelValue, (val) => {
    if (JSON.stringify(val) !== JSON.stringify(config)) {
        Object.assign(config, JSON.parse(JSON.stringify(val)))
        hasChecksum.value = !!config.checksum
        // 如果当前选中失效，重置
        if (selectedIndex.value >= config.rules.length) {
            selectedIndex.value = config.rules.length > 0 ? 0 : -1
        }
    }
}, { deep: true })

// 监听 config 变化并 emit
watch(config, (val) => {
    emit('update:modelValue', val)
}, { deep: true })

// 校验和切换
function handleChecksumToggle(val: boolean) {
    if (val) {
        config.checksum = {
            algorithm: ChecksumAlgorithm.Crc16Modbus,
            range_start: 0,
            range_end_offset: 2,
            big_endian: false
        }
    } else {
        config.checksum = undefined
    }
}

// 添加规则
function addRule() {
    config.rules.push({
        name: `Rule ${config.rules.length + 1}`,
        match_pattern: { type: 'Hex', value: '' },
        action: { type: 'Static', value: '' }
    })
    // 自动选中新规则
    selectedIndex.value = config.rules.length - 1
}

// 删除规则
function removeRule(index: number) {
    config.rules.splice(index, 1)
    if (config.rules.length === 0) {
        selectedIndex.value = -1
    } else if (selectedIndex.value >= index) {
        selectedIndex.value = Math.max(0, selectedIndex.value - 1)
    }
}

// 移动规则
function moveRule(index: number, direction: number) {
    if (index + direction < 0 || index + direction >= config.rules.length) return
    const temp = config.rules[index]
    config.rules[index] = config.rules[index + direction]
    config.rules[index + direction] = temp
    // 保持选中状态跟随
    if (selectedIndex.value === index) {
        selectedIndex.value = index + direction
    }
}

function selectRule(index: number) {
    selectedIndex.value = index
}

// 辅助函数
function getMatchPatternLabel(type: string): string {
    const map: Record<string, string> = {
        'Regex': '正则',
        'Hex': 'Hex',
        'StringContain': '包含',
        'Any': '任意'
    }
    return map[type] || type
}

function getResponseActionLabel(type: string): string {
    const map: Record<string, string> = {
        'Static': '静态(Hex)',
        'StaticString': '静态(Str)',
        'Template': '模板(Hex)',
        'TemplateString': '模板(Str)',
        'Delay': '延迟',
        'None': '无'
    }
    return map[type] || type
}

function getMatchPlaceholder(type: string): string {
    switch (type) {
        case 'Regex': return '输入正则表达式'
        case 'Hex': return '输入十六进制字符串'
        case 'StringContain': return '输入包含的字符串'
        default: return ''
    }
}

function getMatchExample(type: string): string {
    switch (type) {
        case 'Regex': return '示例: ^(01|02).+03$ (匹配以01或02开头, 03结尾)'
        case 'Hex': return '示例: 01 03 00 00 00 01 (十六进制字节序列)'
        case 'StringContain': return '示例: GET /api/data (包含此字符串)'
        default: return ''
    }
}

function getActionPlaceholder(type: string): string {
    switch (type) {
        case 'Static': return '输入响应 Hex 数据'
        case 'StaticString': return '输入响应字符串数据'
        case 'Template': return '输入模板字符串 (结果需为有效Hex)'
        case 'TemplateString': return '输入模板字符串'
        default: return ''
    }
}

function getActionExample(type: string): string {
    switch (type) {
        case 'Static': return '示例: 01 03 04 00 00 00 00'
        case 'StaticString': return '示例: {"status": "ok"}'
        case 'Template': return '示例: 010304{{ value }}'
        case 'TemplateString': return '示例: Current value is: {{ value }}'
        default: return ''
    }
}
</script>

<style lang="scss" scoped>
.rule-editor-container {
    display: flex;
    flex-direction: column;
    gap: 15px;
    height: 100%;
}

.checksum-card {
    flex-shrink: 0;

    .card-title {
        font-weight: 600;
        font-size: 14px;
    }

    .checksum-config {
        padding-top: 5px;
    }

    :deep(.el-card__header) {
        padding: 10px 15px;
        border-bottom: 1px solid var(--border-color);
    }

    :deep(.el-card__body) {
        padding: 10px 15px;
    }
}

.main-editor {
    flex: 1;
    display: flex;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    overflow: hidden;
    background: var(--bg-color-overlay);
    min-height: 400px;
}

/* 左侧列表 */
.rule-list-panel {
    width: 280px;
    border-right: 1px solid var(--border-color);
    display: flex;
    flex-direction: column;
    background: var(--bg-color);

    .list-header {
        padding: 10px 15px;
        border-bottom: 1px solid var(--border-color);
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-weight: 600;
        font-size: 14px;
        background: var(--bg-header);
    }

    .list-content {
        flex: 1;
        overflow-y: auto;
        padding: 10px;
    }

    .rule-item {
        padding: 10px;
        margin-bottom: 8px;
        border-radius: 6px;
        cursor: pointer;
        border: 1px solid transparent;
        background: var(--bg-color-overlay);
        transition: all 0.2s;
        position: relative;

        &:hover {
            background: var(--bg-hover);

            .rule-actions {
                opacity: 1;
            }
        }

        &.active {
            background: var(--primary-light);
            border-color: var(--primary);
            color: var(--primary-text);

            .rule-index {
                color: var(--primary);
            }
        }

        .rule-item-header {
            display: flex;
            align-items: center;
            margin-bottom: 6px;

            .rule-index {
                font-size: 12px;
                color: var(--text-muted);
                margin-right: 8px;
                min-width: 25px;
            }

            .rule-name {
                font-weight: 500;
                font-size: 13px;
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }
        }

        .rule-badges {
            display: flex;
            align-items: center;
            gap: 4px;

            .arrow-icon {
                font-size: 12px;
                color: var(--text-muted);
            }
        }

        .rule-actions {
            position: absolute;
            right: 5px;
            top: 5px;
            opacity: 0;
            transition: opacity 0.2s;
            background: rgba(0, 0, 0, 0.5); // 半透明背景以防文字重叠
            border-radius: 4px;
            padding: 2px;
        }
    }
}

/* 右侧详情 */
.rule-detail-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--bg-color-overlay);

    .detail-content {
        flex: 1;
        overflow-y: auto;
        padding: 20px;
    }

    .detail-header {
        margin-bottom: 20px;

        .rule-name-input {
            font-size: 16px;
            font-weight: 600;
        }
    }

    .empty-selection {
        flex: 1;
        display: flex;
        justify-content: center;
        align-items: center;
        color: var(--text-muted);
    }
}

.help-text {
    font-size: 12px;
    color: var(--text-muted);
    margin-top: 6px;
    display: flex;
    align-items: center;
    gap: 4px;
    line-height: 1.4;

    &.warning {
        color: #e6a23c;
    }
}

/* 自定义滚动条 */
.custom-scrollbar {
    &::-webkit-scrollbar {
        width: 6px;
        height: 6px;
    }

    &::-webkit-scrollbar-track {
        background: transparent;
    }

    &::-webkit-scrollbar-thumb {
        background: rgba(144, 147, 153, 0.3);
        border-radius: 3px;
    }

    &::-webkit-scrollbar-thumb:hover {
        background: rgba(144, 147, 153, 0.5);
    }
}
</style>
