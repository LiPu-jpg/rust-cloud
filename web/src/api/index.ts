// [知识点 #201] Axios 客户端封装
// ----------------------------------------
// 题目：为什么要封装 API 客户端？
//
// 讲解：
// 封装 API 客户端的好处：
// 1. 统一配置（baseURL、headers）
// 2. 类型安全（TypeScript 泛型）
// 3. 错误处理集中
// 4. 易于维护和修改
//
// Axios vs Fetch：
// - Axios: 功能更丰富，自动转换 JSON，拦截器支持
// - Fetch: 原生 API，无依赖，但需要手动处理
//
// 思考：什么时候使用 Fetch 而不是 Axios？
// ----------------------------------------

import axios from 'axios';
import type { ApiResponse, FileInfo, Device, FileRecord, SyncRecord, SyncPlanItem } from '../types';

// [知识点 #202] Axios 实例配置
// ----------------------------------------
// 题目：axios.create() 做了什么？
//
// 讲解：
// axios.create() 创建一个配置好的实例：
// - baseURL: 所有请求的基础 URL
// - headers: 默认请求头
// - timeout: 请求超时时间
//
// Vite 代理会将 /api 转发到后端
//
// 思考：如何在开发环境和生产环境切换 API 地址？
// ----------------------------------------

const api = axios.create({
  baseURL: '/api',
  headers: {
    'Content-Type': 'application/json',
  },
});

// [知识点 #203] API 方法类型定义
// ----------------------------------------
// 题目：为什么要定义返回类型？
//
// 讲解：
// TypeScript 泛型确保类型安全：
// - api.get<ApiResponse<FileInfo[]>>()
// - 编译时检查返回值类型
// - 编辑器智能提示
//
// 这是前后端类型共享的一种方式
//
// 思考：如何实现前后端类型自动生成？
// ----------------------------------------

// 文件 API
export const fileApi = {
  listFiles: (path?: string) =>
    api.get<ApiResponse<FileInfo[]>>('/files', { params: { path } }).then(r => r.data),
  
  getFile: (path: string) =>
    api.get<ApiResponse<FileInfo>>(`/files/${path}`).then(r => r.data),
  
  uploadFile: (path: string, content: string) =>
    api.put<ApiResponse<FileInfo>>(`/files/${path}`, content).then(r => r.data),
  
  deleteFile: (path: string) =>
    api.delete<ApiResponse<boolean>>(`/files/${path}`).then(r => r.data),

  getContent: (path: string) =>
    api.get(`/files/${path}`, { responseType: 'text' }),

  createFolder: (path: string) =>
    api.post<ApiResponse<FileInfo>>('/files', { path }).then(r => r.data),
};

// 设备 API
export const deviceApi = {
  listDevices: () =>
    api.get<ApiResponse<Device[]>>('/devices').then(r => r.data),
  
  registerDevice: (name: string) =>
    api.post<ApiResponse<Device>>('/devices', { name }).then(r => r.data),
  
  heartbeat: (id: string) =>
    api.post<ApiResponse<Device>>(`/devices/${id}/heartbeat`).then(r => r.data),
};

// 版本 API
export const versionApi = {
  listVersions: () =>
    api.get<ApiResponse<FileRecord[]>>('/versions').then(r => r.data),
};

// 同步 API
export const syncApi = {
  getSyncStatus: (fileId: string) =>
    api.get<ApiResponse<SyncRecord[]>>(`/syncs/${fileId}`).then(r => r.data),
  
  createSyncPlan: (localFiles: FileRecord[]) =>
    api.post<ApiResponse<SyncPlanItem[]>>('/sync/plan', { local_files: localFiles }).then(r => r.data),
  
  executeSync: (fileId: string, deviceId: string, action: string) =>
    api.post<ApiResponse<boolean>>('/sync/execute', { 
      file_id: fileId, 
      device_id: deviceId, 
      action 
    }).then(r => r.data),
};

export default api;
