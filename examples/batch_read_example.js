/**
 * 批量读取接口 JavaScript/Node.js 示例
 * 
 * 安装依赖:
 *   npm install axios
 * 
 * 运行:
 *   node batch_read_example.js
 */

const axios = require('axios');

class DeviceClient {
  constructor(baseUrl = 'http://localhost:8080') {
    this.baseUrl = baseUrl;
    this.client = axios.create({
      baseURL: baseUrl,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json'
      }
    });
  }

  /**
   * 批量读取数据
   * @param {Array} items - 读取项列表
   * @returns {Promise<Object>} API 响应
   */
  async batchRead(items) {
    try {
      const response = await this.client.post('/device/batchRead', { items });
      return response.data;
    } catch (error) {
      console.error('批量读取失败:', error.message);
      return {
        state: -1,
        message: `请求失败: ${error.message}`,
        data: []
      };
    }
  }

  /**
   * 批量读取并返回字典格式
   * @param {Array} items - 读取项列表
   * @returns {Promise<Object>} {name: value} 格式
   */
  async batchReadDict(items) {
    const result = await this.batchRead(items);
    const dict = {};
    
    if (result.data) {
      result.data.forEach(item => {
        if (item.success) {
          dict[item.name] = item.value;
        }
      });
    }
    
    return dict;
  }

  /**
   * 批量读取并保留完整元数据
   * @param {Array} items - 读取项列表
   * @returns {Promise<Array>} 包含成功/失败信息的列表
   */
  async batchReadWithMetadata(items) {
    const result = await this.batchRead(items);
    return result.data || [];
  }
}

// ===================================================================
// 示例 1: 基础批量读取
// ===================================================================
async function demo1_basicBatchRead() {
  console.log('\n' + '='.repeat(70));
  console.log('示例 1: 基础批量读取');
  console.log('='.repeat(70) + '\n');

  const client = new DeviceClient();

  const items = [
    { name: '温度传感器', channel_id: 3, addr: 100, type: 'int16' },
    { name: '压力传感器', channel_id: 3, addr: 200, type: 'float32' },
    { name: '流量计', channel_id: 3, addr: 300, type: 'uint32' }
  ];

  console.log('读取配置:');
  items.forEach(item => {
    console.log(`  - ${item.name}: Channel ${item.channel_id}, Addr ${item.addr}, Type ${item.type}`);
  });

  console.log('\n发送请求...');
  const result = await client.batchRead(items);

  console.log(`\n响应状态: ${result.state}`);
  console.log(`响应消息: ${result.message}\n`);
  console.log('读取结果:');

  result.data.forEach(item => {
    if (item.success) {
      console.log(`  ✓ ${item.name}: ${item.value}`);
    } else {
      console.log(`  ✗ ${item.name}: ${item.error || '未知错误'}`);
    }
  });
}

// ===================================================================
// 示例 2: 字典模式（简化访问）
// ===================================================================
async function demo2_dictMode() {
  console.log('\n' + '='.repeat(70));
  console.log('示例 2: 字典模式 - 简化数据访问');
  console.log('='.repeat(70) + '\n');

  const client = new DeviceClient();

  const items = [
    { name: '温度', channel_id: 3, addr: 100, type: 'int16' },
    { name: '压力', channel_id: 3, addr: 200, type: 'float32' },
    { name: '流量', channel_id: 3, addr: 300, type: 'uint32' }
  ];

  console.log('使用字典模式读取...');
  const data = await client.batchReadDict(items);

  console.log('\n可直接通过名称访问数据:');
  console.log('  const temperature = data["温度"] / 10;');
  console.log('  const pressure = data["压力"];');
  console.log('  const flow = data["流量"];\n');

  if (Object.keys(data).length > 0) {
    console.log('实际值:');
    for (const [name, value] of Object.entries(data)) {
      console.log(`  ${name}: ${value}`);
    }
  } else {
    console.log('没有成功读取到数据');
  }
}

// ===================================================================
// 示例 3: 数据处理与转换
// ===================================================================
async function demo3_dataProcessing() {
  console.log('\n' + '='.repeat(70));
  console.log('示例 3: 数据处理与转换');
  console.log('='.repeat(70) + '\n');

  const client = new DeviceClient();

  // 定义带缩放系数的配置
  const itemsConfig = [
    { name: '温度', channel_id: 3, addr: 100, type: 'int16', scale: 0.1, unit: '°C' },
    { name: '压力', channel_id: 3, addr: 200, type: 'float32', scale: 1, unit: 'Pa' },
    { name: '流量', channel_id: 3, addr: 300, type: 'uint32', scale: 0.001, unit: 'm³/h' },
    { name: '液位', channel_id: 3, addr: 400, type: 'uint16', scale: 0.01, unit: 'm' }
  ];

  // 提取读取项（不包含 scale 和 unit）
  const readItems = itemsConfig.map(({ name, channel_id, addr, type }) => ({
    name, channel_id, addr, type
  }));

  console.log('读取原始数据...');
  const result = await client.batchRead(readItems);

  console.log('\n原始数据 → 工程数据:');
  console.log('-'.repeat(70));

  result.data.forEach(item => {
    if (!item.success) return;

    const config = itemsConfig.find(c => c.name === item.name);
    const rawValue = item.value;
    const scaledValue = rawValue * config.scale;

    console.log(`  ${config.name.padEnd(10)}: ${String(rawValue).padStart(12)} (原始) → ${scaledValue.toFixed(3).padStart(12)} ${config.unit}`);
  });
}

// ===================================================================
// 示例 4: 实时监控
// ===================================================================
class RealtimeMonitor {
  constructor(client, items, interval = 1000) {
    this.client = client;
    this.items = items;
    this.interval = interval;
    this.timer = null;
  }

  async start() {
    console.log('='.repeat(70));
    console.log('实时监控开始 (按 Ctrl+C 停止)');
    console.log('='.repeat(70) + '\n');

    this.timer = setInterval(async () => {
      const timestamp = new Date().toLocaleString('zh-CN');
      const result = await this.client.batchRead(this.items);

      console.log(`\n[${timestamp}] 状态: ${result.message}`);
      console.log('-'.repeat(70));

      result.data.forEach(item => {
        const status = item.success ? '✓' : '✗';
        const name = item.name.padEnd(20);

        if (item.success) {
          console.log(`  ${status} ${name} = ${item.value}`);
        } else {
          console.log(`  ${status} ${name} - 错误: ${item.error || '未知错误'}`);
        }
      });
    }, this.interval);
  }

  stop() {
    if (this.timer) {
      clearInterval(this.timer);
      console.log('\n\n监控停止');
    }
  }
}

async function demo4_realtimeMonitoring() {
  console.log('\n' + '='.repeat(70));
  console.log('示例 4: 实时监控演示');
  console.log('='.repeat(70) + '\n');

  const client = new DeviceClient();

  const items = [
    { name: '温度', channel_id: 3, addr: 100, type: 'int16' },
    { name: '压力', channel_id: 3, addr: 200, type: 'float32' },
    { name: '流量', channel_id: 3, addr: 300, type: 'uint32' },
    { name: '泵状态', channel_id: 3, addr: 0, type: 'bool' }
  ];

  console.log('监控点位:');
  items.forEach(item => console.log(`  - ${item.name}`));
  console.log('\n更新间隔: 2 秒\n');

  const monitor = new RealtimeMonitor(client, items, 2000);
  monitor.start();

  // 监控 10 秒后自动停止
  setTimeout(() => {
    monitor.stop();
    console.log('\n演示完成');
  }, 10000);
}

// ===================================================================
// 示例 5: 错误处理
// ===================================================================
async function demo5_errorHandling() {
  console.log('\n' + '='.repeat(70));
  console.log('示例 5: 错误处理演示');
  console.log('='.repeat(70) + '\n');

  const client = new DeviceClient();

  // 故意包含错误配置
  const items = [
    { name: '正常点位', channel_id: 3, addr: 100, type: 'int16' },
    { name: '不存在的通道', channel_id: 999, addr: 100, type: 'int16' },
    { name: '错误地址', channel_id: 3, addr: 99999, type: 'int16' }
  ];

  console.log('测试配置（含错误项）:');
  items.forEach(item => console.log(`  - ${item.name}`));

  console.log('\n执行批量读取...');
  const result = await client.batchRead(items);

  console.log(`\n${result.message}\n`);
  console.log('详细结果:');

  let successCount = 0;
  let errorCount = 0;

  result.data.forEach(item => {
    if (item.success) {
      successCount++;
      console.log(`  ✓ ${item.name.padEnd(20)} = ${item.value}`);
    } else {
      errorCount++;
      console.log(`  ✗ ${item.name.padEnd(20)} - ${item.error || '未知错误'}`);
    }
  });

  console.log(`\n统计: 成功 ${successCount}, 失败 ${errorCount}`);
}

// ===================================================================
// 示例 6: JSON 数据导出
// ===================================================================
async function demo6_jsonExport() {
  console.log('\n' + '='.repeat(70));
  console.log('示例 6: 数据导出为 JSON');
  console.log('='.repeat(70) + '\n');

  const client = new DeviceClient();

  const items = [
    { name: '温度', channel_id: 3, addr: 100, type: 'int16' },
    { name: '压力', channel_id: 3, addr: 200, type: 'float32' },
    { name: '流量', channel_id: 3, addr: 300, type: 'uint32' }
  ];

  console.log('读取数据...');
  const result = await client.batchRead(items);

  // 构建导出数据
  const exportData = {
    timestamp: new Date().toISOString(),
    device: '设备1',
    readings: {}
  };

  result.data.forEach(item => {
    if (item.success) {
      exportData.readings[item.name] = {
        value: item.value,
        success: true
      };
    } else {
      exportData.readings[item.name] = {
        error: item.error,
        success: false
      };
    }
  });

  console.log('\n导出的 JSON 数据:');
  console.log(JSON.stringify(exportData, null, 2));

  // 可以保存到文件
  // const fs = require('fs');
  // fs.writeFileSync('data_export.json', JSON.stringify(exportData, null, 2));
}

// ===================================================================
// 主程序
// ===================================================================
async function main() {
  console.log(`
╔══════════════════════════════════════════════════════════════════════╗
║              批量读取接口 JavaScript 示例程序                         ║
║                                                                      ║
║  请确保:                                                              ║
║  1. dm-rust 服务已启动 (cargo run)                                   ║
║  2. 配置文件中有对应的通道（channel_id: 3 等）                        ║
║  3. 已安装 axios: npm install axios                                  ║
╚══════════════════════════════════════════════════════════════════════╝
  `);

  // 检查服务是否运行
  try {
    await axios.get('http://localhost:8080/', { timeout: 2000 });
    console.log('✓ 服务连接正常\n');
  } catch (error) {
    console.log('✗ 无法连接到服务，请先启动 dm-rust');
    console.log('  运行: cd dm-rust && cargo run\n');
    return;
  }

  // 运行所有演示
  try {
    await demo1_basicBatchRead();
    await new Promise(resolve => setTimeout(resolve, 1000));

    await demo2_dictMode();
    await new Promise(resolve => setTimeout(resolve, 1000));

    await demo3_dataProcessing();
    await new Promise(resolve => setTimeout(resolve, 1000));

    await demo5_errorHandling();
    await new Promise(resolve => setTimeout(resolve, 1000));

    await demo6_jsonExport();
    await new Promise(resolve => setTimeout(resolve, 1000));

    // 实时监控示例（运行 10 秒）
    await demo4_realtimeMonitoring();

  } catch (error) {
    console.error('\n错误:', error.message);
  }
}

// 运行主程序
if (require.main === module) {
  main().catch(console.error);
}

// 导出供其他模块使用
module.exports = {
  DeviceClient,
  RealtimeMonitor
};
