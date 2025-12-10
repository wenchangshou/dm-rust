#!/bin/bash

echo "测试超时错误处理"
echo "=================="
echo ""

echo "发送命令: channel_on 通道1"
echo "预期: 3秒后返回超时错误"
echo ""

time curl -X POST http://localhost:8080/device/callMethod \
  -H 'Content-Type: application/json' \
  -d '{"channel_id":1,"method_name":"channel_on","arguments":{"channel":1}}'

echo ""
echo ""
echo "完成!"
