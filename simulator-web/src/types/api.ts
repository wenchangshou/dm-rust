/** API 响应基础结构 */
export interface ApiResponse<T = unknown> {
  state: number
  message: string
  data?: T
}

/** 判断响应是否成功 */
export function isSuccess(response: ApiResponse): boolean {
  return response.state === 0
}
