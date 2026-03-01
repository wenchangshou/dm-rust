export type LocaleCode = 'zh-CN' | 'en-US'

type MessageValue = string | Messages
interface Messages {
  [key: string]: MessageValue
}

export const messages: Record<LocaleCode, Messages> = {
  'zh-CN': {
    common: {
      save: '保存配置',
      reload: '重新加载',
      saving: '保存中...',
      loading: '加载中...',
      add: '新增',
      edit: '编辑',
      delete: '删除',
      cancel: '取消',
      create: '创建',
      update: '更新',
      actions: '操作',
      enabled: '启用',
      disabled: '禁用',
      empty: '暂无数据'
    },
    sidebar: {
      title: 'DM-Rust 配置中心',
      channels: '通道管理',
      nodes: '设备节点',
      scenes: '场景编排',
      language: '语言',
      chinese: '中文',
      english: 'English'
    },
    overview: {
      title: '系统总览',
      desc: '集中管理通信通道、设备节点、场景和系统参数。',
      webPort: 'Web 服务端口',
      channels: '通道数',
      nodes: '节点数',
      scenes: '场景数'
    },
    toast: {
      loaded: '配置加载成功',
      loadFailed: '加载失败：{{message}}',
      connectionError: '连接异常：{{message}}',
      saved: '配置已保存，重启服务后生效',
      saveFailed: '保存失败：{{message}}',
      validationError: '校验失败：{{message}}'
    },
    channels: {
      title: '通道管理',
      desc: '通道定义了系统与硬件设备的通信方式，每个通道绑定一种协议。',
      add: '新增通道',
      empty: '尚未配置通道',
      editTitle: '编辑通道',
      createTitle: '新建通道',
      channelId: '通道 ID',
      description: '描述',
      protocol: '协议',
      protocolSelect: '请选择协议',
      protocolArgs: '协议参数',
      schemaMissing: '未找到协议 Schema：{{name}}',
      status: '状态',
      noProtocol: '未配置协议',
      confirmDelete: '确认删除通道 {{id}} 吗？',
      defaultName: '通道 {{id}}',
      validation: {
        idRequired: '通道 ID 必须大于 0',
        idDuplicate: '通道 ID 不能重复',
        protocolRequired: '请选择协议'
      }
    },
    nodes: {
      title: '设备节点',
      desc: '节点代表可控设备。节点通过通道 ID 关联通信通道，并在通道内使用设备 ID 定位。',
      add: '新增节点',
      empty: '暂无节点，请先创建通道再添加节点。',
      groupTitle: '通道 {{id}}',
      globalId: '全局 ID',
      deviceId: '设备 ID',
      alias: '别名',
      channel: '所属通道',
      nodeIdInChannel: '设备 ID（通道内）',
      editTitle: '编辑节点',
      createTitle: '新建节点',
      confirmDelete: '确认删除节点 “{{name}}” 吗？',
      validation: {
        globalIdRequired: '全局 ID 必须大于 0',
        globalIdDuplicate: '全局 ID 不能重复',
        aliasRequired: '别名不能为空',
        channelRequired: '请先创建至少一个通道'
      }
    },
    scenes: {
      title: '场景编排',
      desc: '场景由多个步骤组成，每个步骤可设置目标节点、目标值及延时。',
      add: '新增场景',
      empty: '尚未配置场景',
      createTitle: '新建场景',
      editTitle: '编辑场景',
      sceneName: '场景名称',
      steps: '步骤',
      stepCount: '{{count}} 个步骤',
      addStep: '新增步骤',
      device: '设备（全局 ID）',
      value: '目标值',
      delay: '延时（ms）',
      noSteps: '尚未添加步骤',
      more: '+{{count}} 个',
      confirmDelete: '确认删除场景 “{{name}}” 吗？',
      validation: {
        nameRequired: '场景名称不能为空',
        noNode: '当前没有可用节点，无法添加步骤'
      }
    }
  },
  'en-US': {
    common: {
      save: 'Save Config',
      reload: 'Reload',
      saving: 'Saving...',
      loading: 'Loading...',
      add: 'Add',
      edit: 'Edit',
      delete: 'Delete',
      cancel: 'Cancel',
      create: 'Create',
      update: 'Update',
      actions: 'Actions',
      enabled: 'Enabled',
      disabled: 'Disabled',
      empty: 'No data'
    },
    sidebar: {
      title: 'DM-Rust Config Center',
      channels: 'Channels',
      nodes: 'Nodes',
      scenes: 'Scenes',
      language: 'Language',
      chinese: '中文',
      english: 'English'
    },
    overview: {
      title: 'Overview',
      desc: 'Manage channels, device nodes, scenes, and system settings in one place.',
      webPort: 'Web Server Port',
      channels: 'Channels',
      nodes: 'Nodes',
      scenes: 'Scenes'
    },
    toast: {
      loaded: 'Configuration loaded',
      loadFailed: 'Load failed: {{message}}',
      connectionError: 'Connection error: {{message}}',
      saved: 'Saved. Restart service to apply changes',
      saveFailed: 'Save failed: {{message}}',
      validationError: 'Validation failed: {{message}}'
    },
    channels: {
      title: 'Channels',
      desc: 'Channels define how the system communicates with hardware. Each channel binds to one protocol.',
      add: 'Add Channel',
      empty: 'No channels configured yet.',
      editTitle: 'Edit Channel',
      createTitle: 'New Channel',
      channelId: 'Channel ID',
      description: 'Description',
      protocol: 'Protocol',
      protocolSelect: 'Select protocol',
      protocolArgs: 'Protocol Arguments',
      schemaMissing: 'Schema not found: {{name}}',
      status: 'Status',
      noProtocol: 'No protocol',
      confirmDelete: 'Delete channel {{id}}?',
      defaultName: 'Channel {{id}}',
      validation: {
        idRequired: 'Channel ID must be greater than 0',
        idDuplicate: 'Channel ID must be unique',
        protocolRequired: 'Please select a protocol'
      }
    },
    nodes: {
      title: 'Device Nodes',
      desc: 'A node is a controllable device linked to a channel by Channel ID and addressed by Device ID in that channel.',
      add: 'Add Node',
      empty: 'No nodes. Create channels first.',
      groupTitle: 'Channel {{id}}',
      globalId: 'Global ID',
      deviceId: 'Device ID',
      alias: 'Alias',
      channel: 'Channel',
      nodeIdInChannel: 'Device ID (in channel)',
      editTitle: 'Edit Node',
      createTitle: 'New Node',
      confirmDelete: 'Delete node "{{name}}"?',
      validation: {
        globalIdRequired: 'Global ID must be greater than 0',
        globalIdDuplicate: 'Global ID must be unique',
        aliasRequired: 'Alias is required',
        channelRequired: 'Create at least one channel first'
      }
    },
    scenes: {
      title: 'Scenes',
      desc: 'A scene contains multiple steps. Each step can set node target value and delay.',
      add: 'Add Scene',
      empty: 'No scenes configured.',
      createTitle: 'New Scene',
      editTitle: 'Edit Scene',
      sceneName: 'Scene Name',
      steps: 'Steps',
      stepCount: '{{count}} steps',
      addStep: 'Add Step',
      device: 'Device (Global ID)',
      value: 'Value',
      delay: 'Delay (ms)',
      noSteps: 'No steps yet',
      more: '+{{count}} more',
      confirmDelete: 'Delete scene "{{name}}"?',
      validation: {
        nameRequired: 'Scene name is required',
        noNode: 'No node available for steps'
      }
    }
  }
}
