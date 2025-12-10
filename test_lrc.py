#!/usr/bin/env python3

data = '00100B00001001'
# 转换为字节
bytes_list = [int(data[i:i+2], 16) for i in range(0, len(data), 2)]
print(f'数据: {data}')
print(f'字节: {bytes_list}')
print(f'字节十六进制: {[hex(b) for b in bytes_list]}')
# 计算和
sum_val = sum(bytes_list) & 0xFF
print(f'和 (低8位): {sum_val} (0x{sum_val:02X})')
# LRC = -sum 的补码
lrc = (256 - sum_val) & 0xFF
print(f'LRC (256-sum): {lrc} (0x{lrc:02X})')
# LRC = (!sum) + 1
lrc2 = ((0xFF - sum_val) + 1) & 0xFF
print(f'LRC (!sum+1): {lrc2} (0x{lrc2:02X})')
print(f'\n期望 LRC: 0x3E ({0x3E})')
