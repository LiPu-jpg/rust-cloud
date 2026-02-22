// [知识点 #212] TypeScript 类型定义
// ----------------------------------------
// 题目：为什么要定义接口？
//
// 讲解：
// TypeScript 接口的好处：
// 1. 类型检查 - 编译时发现错误
// 2. 智能提示 - 编辑器提供自动补全
// 3. 文档作用 - 接口即文档
// 4. 代码重构 - 重构时更安全
//
// 这些类型与后端 Rust 结构体对应
// 实现了前后端类型的一致性
// ----------------------------------------

/**
 * RustCloud API 类型定义
 * 
 * 与后端 Rust 结构体对应：
 * - ApiResponse<T> -> 后端 ApiResponse
 * - FileInfo -> 后端 FileInfo
 * - Device -> 后端 DeviceRecord
 */

// [知识点 #213] 泛型接口
// ----------------------------------------
// 题目：ApiResponse<T = unknown> 的 T 是什么？
//
// 讲解：
// 泛型 T 表示 data 字段的类型：
// - ApiResponse<FileInfo[]> - 文件列表响应
// - ApiResponse<Device> - 单个设备响应
// - T = unknown 是默认值
//
// 这样一个接口可以适配多种响应类型
// ----------------------------------------

export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified?: string;
  hash?: string;
  version?: number;
  content?: string;
}

export interface Device {
  id: string;
  name: string;
  last_seen: string;
}

export interface FileRecord {
  id: string;
  path: string;
  hash?: string;
  size: number;
  version: number;
  created_at: string;
  updated_at: string;
}

export interface SyncRecord {
  id: string;
  device_id: string;
  file_id: string;
  sync_status: SyncStatus;
  last_sync_at: string;
}

// [知识点 #214] TypeScript 类型别名
// ----------------------------------------
// 题目：type 和 interface 有什么区别？
//
// 讲解：
// type 和 interface 都可以定义类型：
// - interface: 可扩展，支持声明合并
// - type: 更灵活，可定义联合类型、交叉类型
//
// 这里用 type 定义简单的联合类型
// ----------------------------------------

export type SyncStatus = 'PENDING' | 'SYNCING' | 'COMPLETED' | 'FAILED';

export interface BreadcrumbItem {
  name: string;
  path: string;
}

export interface SearchResult {
  files: FileInfo[];
  total: number;
}
