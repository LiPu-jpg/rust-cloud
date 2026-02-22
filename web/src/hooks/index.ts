// [知识点 #204] React Query Hooks 封装
// ----------------------------------------
// 题目：为什么需要自定义 Hooks？
//
// 讲解：
// 自定义 Hooks 的好处：
// 1. 逻辑复用 - 相同的请求逻辑可以复用
// 2. 关注点分离 - 数据获取逻辑与 UI 分离
// 3. 测试友好 - 可以单独测试 Hook
// 4. 代码简洁 - 组件更简洁
//
// 思考：自定义 Hooks 和普通函数有什么区别？
// ----------------------------------------

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { fileApi, deviceApi, versionApi, syncApi } from '../api';

// [知识点 #205] useQuery 数据获取
// ----------------------------------------
// 题目：useQuery 的参数是什么？
//
// 讲解：
// useQuery 接收两个主要参数：
// - queryKey: 缓存键，用于标识查询
// - queryFn: 实际获取数据的异步函数
//
// React Query 自动处理：
// - 缓存数据
// - 后台刷新
// - 加载/错误状态
// - 竞态条件
//
// 思考：queryKey 有什么作用？
// ----------------------------------------

// [知识点 #206] useMutation 数据修改
// ----------------------------------------
// 题目：为什么上传/删除使用 useMutation 而不是 useQuery？
//
// 讲解：
// useMutation 用于：
// - 创建、更新、删除操作（副作用）
// - 不期望缓存结果
// - 需要乐观更新
//
// 主要属性：
// - mutationFn: 执行修改的函数
// - onSuccess: 成功后回调（这里用于刷新缓存）
// - onError: 失败后回调
//
// 思考：如何实现乐观更新？
// ----------------------------------------

// 文件相关 Hooks

// [知识点 #207] queryKey 缓存策略
// ----------------------------------------
// 题目：queryKey 为什么用数组？
//
// 讲解：
// queryKey 使用数组的原因：
// - 支持依赖查询（如用户 ID 变化时自动刷新）
// - ['files', path] 当 path 变化时自动重新获取
// - 便于缓存失效（invalidateQueries）
//
// 缓存失效：
// - invalidateQueries: 让指定 key 的查询失效
// - 失效后 React Query 会自动重新获取
//
// 思考：如何精细控制缓存？
// ----------------------------------------

export const useFiles = (path?: string) => {
  return useQuery({
    queryKey: ['files', path],
    queryFn: () => fileApi.listFiles(path),
  });
};

export const useUploadFile = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: ({ path, content }: { path: string; content: string }) =>
      fileApi.uploadFile(path, content),
    onSuccess: () => {
      // [知识点 #208] 缓存失效策略
      // ----------------------------------------
      // 题目：为什么要在成功后 invalidateQueries？
      //
      // 讲解：
      // 修改数据后，需要让相关查询失效：
      // - 上传文件后，文件列表需要刷新
      // - 删除文件后，版本列表也需要刷新
      //
      // 这样 React Query 会自动重新获取最新数据
      // 无需手动触发 refetch
      // ----------------------------------------
      queryClient.invalidateQueries({ queryKey: ['files'] });
      queryClient.invalidateQueries({ queryKey: ['versions'] });
    },
  });
};

export const useDeleteFile = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (path: string) => fileApi.deleteFile(path),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['files'] });
      queryClient.invalidateQueries({ queryKey: ['versions'] });
    },
  });
};

// [知识点 #209] 条件查询
// ----------------------------------------
// 题目：enabled: !!path 是什么意思？
//
// 讲解：
// enabled 属性控制是否执行查询：
// - enabled: true（默认）：立即执行
// - enabled: false：暂停执行
// - enabled: !!path：path 有值时才执行
//
// 这用于实现条件查询，避免无效请求
// ----------------------------------------

export const useFileContent = (path: string | null) => {
  return useQuery({
    queryKey: ['file-content', path],
    queryFn: () => path ? fileApi.getContent(path) : Promise.resolve(null),
    enabled: !!path,
  });
};

// 设备相关 Hooks

export const useDevices = () => {
  return useQuery({
    queryKey: ['devices'],
    queryFn: () => deviceApi.listDevices(),
  });
};

export const useRegisterDevice = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (name: string) => deviceApi.registerDevice(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['devices'] });
    },
  });
};

export const useDeviceHeartbeat = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: (id: string) => deviceApi.heartbeat(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['devices'] });
    },
  });
};

// 版本相关 Hooks

export const useVersions = () => {
  return useQuery({
    queryKey: ['versions'],
    queryFn: () => versionApi.listVersions(),
  });
};

// 同步相关 Hooks

export const useSyncStatus = (fileId: string | null) => {
  return useQuery({
    queryKey: ['sync-status', fileId],
    queryFn: () => fileId ? syncApi.getSyncStatus(fileId) : Promise.resolve(null),
    enabled: !!fileId,
  });
};
