<template>
  <el-dialog title="创建模拟器" :model-value="visible" @update:model-value="emit('update:visible', $event)" width="600px"
    class="create-dialog">
    <el-tabs v-model="activeTab" class="custom-tabs">
      <el-tab-pane label="自定义创建" name="custom">
        <el-form ref="customFormRef" :model="customForm" :rules="customRules" label-width="100px" class="dialog-form">
          <el-form-item label="名称" prop="name">
            <el-input v-model="customForm.name" placeholder="请输入模拟器名称" />
          </el-form-item>

          <el-form-item label="描述" prop="description">
            <el-input v-model="customForm.description" type="textarea" placeholder="请输入描述 (可选)" :rows="2" />
          </el-form-item>

          <el-form-item label="协议" prop="protocol">
            <el-select v-model="customForm.protocol" placeholder="请选择协议" style="width: 100%">
              <el-option v-for="p in protocols" :key="p.name" :label="`${p.name} - ${p.description}`" :value="p.name" />
            </el-select>
          </el-form-item>

          <el-form-item label="传输协议" prop="transport">
            <el-radio-group v-model="customForm.transport">
              <el-radio label="tcp">TCP</el-radio>
              <el-radio label="udp">UDP</el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item label="绑定地址" prop="bind_addr">
            <el-input v-model="customForm.bind_addr" placeholder="0.0.0.0" />
          </el-form-item>

          <el-form-item label="端口" prop="port">
            <el-input-number v-model="customForm.port" :min="1" :max="65535" style="width: 100%" />
          </el-form-item>

          <el-form-item label="自动启动">
            <el-switch v-model="customForm.auto_start" />
          </el-form-item>

          <!-- 自定义协议规则编辑器 -->
          <div v-if="customForm.protocol === 'custom' && customForm.protocol_config" class="rule-editor-container">
            <RuleEditor v-model="customForm.protocol_config" />
          </div>
        </el-form>
      </el-tab-pane>

      <el-tab-pane label="从模板创建" name="template">
        <div class="template-header">
          <span class="label">选择模板</span>
          <el-button type="primary" link @click="showTemplateManager = true">
            管理模板
          </el-button>
        </div>

        <el-form ref="templateFormRef" :model="templateForm" :rules="templateRules" label-width="100px"
          class="dialog-form">
          <el-form-item label="选择模板" prop="template_id">
            <el-select v-model="templateForm.template_id" placeholder="请选择模板" style="width: 100%"
              no-data-text="暂无模板，请先创建或使用自定义创建">
              <el-option v-for="t in templates" :key="t.id" :label="t.name" :value="t.id">
                <div class="template-option">
                  <span>{{ t.name }}</span>
                  <el-tag size="small" type="info">{{ t.protocol }}</el-tag>
                </div>
              </el-option>
            </el-select>
          </el-form-item>

          <div v-if="selectedTemplate" class="template-info">
            <p>{{ selectedTemplate.description || '无描述' }}</p>
          </div>

          <el-form-item label="名称" prop="name">
            <el-input v-model="templateForm.name" placeholder="请输入模拟器名称" />
          </el-form-item>

          <el-form-item label="绑定地址" prop="bind_addr">
            <el-input v-model="templateForm.bind_addr" placeholder="0.0.0.0" />
          </el-form-item>

          <el-form-item label="端口" prop="port">
            <el-input-number v-model="templateForm.port" :min="1" :max="65535" style="width: 100%" />
          </el-form-item>
        </el-form>
      </el-tab-pane>
    </el-tabs>

    <template #footer>
      <el-button @click="emit('update:visible', false)">取消</el-button>
      <el-button type="primary" :loading="loading" @click="handleSubmit">
        创建
      </el-button>
    </template>

    <TemplateManagerDialog v-model:visible="showTemplateManager" @changed="fetchTemplates" />
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue'
import type { FormInstance, FormRules } from 'element-plus'
import { useSimulatorStore } from '@/stores/simulator'
import type { ProtocolInfo, CreateSimulatorRequest, SimulatorTemplate, CreateFromTemplateRequest } from '@/types/simulator'
import * as simulatorApi from '@/api/simulator'
import TemplateManagerDialog from './TemplateManagerDialog.vue'
import RuleEditor from './RuleEditor.vue'

const props = defineProps<{
  visible: boolean
  protocols: ProtocolInfo[]
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  'created': []
}>()

const store = useSimulatorStore()
const customFormRef = ref<FormInstance>()
const templateFormRef = ref<FormInstance>()
const loading = ref(false)
const activeTab = ref('custom')
const showTemplateManager = ref(false)
const templates = ref<SimulatorTemplate[]>([])

// 自定义创建表单
const customForm = reactive<CreateSimulatorRequest>({
  name: '',
  description: '',
  protocol: '',
  transport: 'tcp',
  bind_addr: '0.0.0.0',
  port: 5000,
  auto_start: true,
  protocol_config: undefined
})

const customRules: FormRules = {
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  protocol: [{ required: true, message: '请选择协议', trigger: 'change' }],
  transport: [{ required: true, message: '请选择传输协议', trigger: 'change' }],
  port: [{ required: true, message: '请输入端口', trigger: 'blur' }],
}

// ... existing code ...

// 监听协议变化，自动设置默认端口和初始化配置
watch(() => customForm.protocol, (protocol) => {
  const p = props.protocols.find(p => p.name === protocol)
  if (p) {
    customForm.port = p.default_port
  }

  if (protocol === 'custom') {
    if (!customForm.protocol_config) {
      customForm.protocol_config = {
        name: 'MyProtocol',
        description: 'Custom Protocol',
        default_port: 5000,
        rules: []
      }
    }
  } else {
    customForm.protocol_config = undefined
  }
})
// 模板创建表单
const templateForm = reactive<CreateFromTemplateRequest>({
  template_id: '',
  name: '',
  bind_addr: '0.0.0.0',
  port: 5000
})

const templateRules: FormRules = {
  template_id: [{ required: true, message: '请选择模板', trigger: 'change' }],
  name: [{ required: true, message: '请输入名称', trigger: 'blur' }],
  port: [{ required: true, message: '请输入端口', trigger: 'blur' }],
}

const selectedTemplate = computed(() => {
  return templates.value.find(t => t.id === templateForm.template_id)
})

// 监听 tab 切换获取模板
watch(activeTab, (tab) => {
  if (tab === 'template') {
    fetchTemplates()
  }
})

// 监听模板选择，自动填充信息
watch(() => templateForm.template_id, (id) => {
  const template = templates.value.find(t => t.id === id)
  if (template) {
    if (!templateForm.name) {
      templateForm.name = `${template.name}_1`
    }
    // 尝试从模板 config 中获取端口 (如果包含) - 但 SimulatorTemplate config 可能是任意 json
    // 使用协议默认端口作为 fallback
    const proto = props.protocols.find(p => p.name === template.protocol)
    if (proto && templateForm.port === 5000) { // 只有默认值才覆盖
      templateForm.port = proto.default_port
    }
  }
})

async function fetchTemplates() {
  try {
    templates.value = await simulatorApi.listTemplates()
  } catch (e) {
    console.error(e)
  }
}

// 监听协议变化，自动设置默认端口
watch(() => customForm.protocol, (protocol) => {
  const p = props.protocols.find(p => p.name === protocol)
  if (p) {
    customForm.port = p.default_port
  }
})

// 对话框关闭时重置表单
watch(() => props.visible, (visible) => {
  if (!visible) {
    customFormRef.value?.resetFields()
    templateFormRef.value?.resetFields()
    activeTab.value = 'custom'
  }
})

async function handleSubmit() {
  loading.value = true
  try {
    if (activeTab.value === 'custom') {
      await customFormRef.value?.validate()
      await store.createSimulator(customForm)
    } else {
      await templateFormRef.value?.validate()
      await simulatorApi.createFromTemplate(templateForm)
      // 需要刷新 store 列表，因为 store action createSimulator 已经刷新了，但 createFromTemplate 不在 store 中
      await store.fetchSimulators()
    }
    emit('created')
  } catch {
    // 验证失败或请求失败
  } finally {
    loading.value = false
  }
}
</script>

<style lang="scss" scoped>
.template-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;

  .label {
    font-size: 14px;
    font-weight: 500;
  }
}

.template-option {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.template-info {
  background: var(--bg-input);
  padding: 12px;
  border-radius: 8px;
  margin-bottom: 20px;
  font-size: 13px;
  color: var(--text-secondary);
}

.dialog-form {
  padding-top: 20px;
}
</style>
