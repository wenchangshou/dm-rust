import axios from 'axios'
import type { ApiResponse } from '@/types/api'
import { ElMessage } from 'element-plus'

const api = axios.create({
  baseURL: '/api/tcp-simulator',
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
})

// 响应拦截器
api.interceptors.response.use(
  (response) => {
    const data = response.data as ApiResponse
    if (data.state !== 0) {
      ElMessage.error(data.message || '操作失败')
      return Promise.reject(new Error(data.message))
    }
    return response
  },
  (error) => {
    ElMessage.error(error.message || '网络错误')
    return Promise.reject(error)
  }
)

export default api
