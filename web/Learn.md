# RustCloud Web 前端学习指引

本文档为前端学习者提供阅读 RustCloud Web 源码的最佳路径。

## 技术栈概览

| 技术 | 作用 | 知识点 |
|------|------|--------|
| React 18 | UI 框架 | 组件化、Hooks |
| TypeScript | 类型安全 | 接口、泛型 |
| Vite | 构建工具 | 开发服务器、热更新 |
| TanStack Query | 数据获取 | 缓存、后台刷新 |
| Axios | HTTP 客户端 | API 封装 |
| Tailwind CSS | 样式 | 原子化 CSS |

---

## 推荐学习顺序

### 阶段一：项目入口 (1小时)

| 文件 | 知识点 | 核心概念 |
|------|--------|----------|
| `src/main.tsx` | React DOM 渲染 | 入口文件 |
| `src/App.tsx` | #210-#211 | QueryClientProvider、全局配置 |

### 阶段二：类型定义 (1小时)

| 文件 | 知识点 | 核心概念 |
|------|--------|----------|
| `src/types/index.ts` | #212-#214 | 接口定义、泛型、type vs interface |

### 阶段三：API 客户端 (2小时)

| 文件 | 知识点 | 核心概念 |
|------|--------|----------|
| `src/api/index.ts` | #201-#203 | Axios 封装、类型安全 |

### 阶段四：数据获取 (2小时)

| 文件 | 知识点 | 核心概念 |
|------|--------|----------|
| `src/hooks/index.ts` | #204-#209 | useQuery、useMutation、缓存策略 |

### 阶段五：组件设计 (3小时)

| 文件 | 知识点 | 核心概念 |
|------|--------|----------|
| `src/components/Layout.tsx` | 页面布局 | React 组件组合 |
| `src/components/FileList.tsx` | 文件列表 | 状态管理、事件处理 |
| `src/components/FilePreview.tsx` | 文件预览 | 条件渲染、useEffect |
| `src/components/Breadcrumbs.tsx` | 面包屑 | 列表渲染 |

### 阶段六：页面逻辑 (2小时)

| 文件 | 知识点 | 核心概念 |
|------|--------|----------|
| `src/pages/FileManager.tsx` | 文件管理 | 组合所有组件和 hooks |
| `src/pages/DeviceManager.tsx` | 设备管理 | 状态过滤、计算属性 |
| `src/pages/VersionManager.tsx` | 版本历史 | 数据分组、统计 |

---

## 知识点详解

### #201 Axios 客户端封装

```typescript
// 创建配置好的 Axios 实例
const api = axios.create({
  baseURL: '/api',  // Vite 代理到后端
  headers: { 'Content-Type': 'application/json' }
});

// 使用泛型指定返回类型
api.get<ApiResponse<FileInfo[]>>('/files')
```

### #204 React Query 基础

```typescript
// 数据查询
const { data, isLoading, error } = useQuery({
  queryKey: ['files', path],
  queryFn: () => api.get('/files')
});

// 数据修改
const mutation = useMutation({
  mutationFn: (data) => api.post('/files', data),
  onSuccess: () => queryClient.invalidateQueries(['files'])
});
```

### #212 TypeScript 接口

```typescript
// 定义 API 响应类型
interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

// 使用泛型
type FilesResponse = ApiResponse<FileInfo[]>;
```

---

## 开发命令

```bash
# 安装依赖
npm install

# 开发模式
npm run dev

# 构建生产版本
npm run build

# 类型检查
npm run build

# 预览构建
npm run preview
```

---

## 项目结构

```
web/src/
├── api/           # API 客户端封装
│   └── index.ts   # axios 实例和 API 方法
├── components/    # React 组件
│   ├── Layout.tsx       # 页面布局
│   ├── Navigation.tsx   # 导航栏
│   ├── FileList.tsx     # 文件列表
│   ├── FileUpload.tsx   # 文件上传
│   ├── FilePreview.tsx  # 文件预览
│   ├── Breadcrumbs.tsx  # 面包屑导航
│   └── DeviceRegistration.tsx # 设备注册
├── hooks/         # 自定义 Hooks
│   └── index.ts   # React Query hooks
├── pages/         # 页面组件
│   ├── FileManager.tsx    # 文件管理页
│   ├── DeviceManager.tsx # 设备管理页
│   └── VersionManager.tsx # 版本历史页
├── types/         # TypeScript 类型
│   └── index.ts   # 接口定义
├── App.tsx        # 根组件
└── main.tsx       # 入口文件
```

---

## 常见问题

### Q: 为什么用 React Query 而不是 useEffect？

- 自动缓存和去重
- 后台自动刷新
- 更好的加载/错误状态管理
- 避免请求竞态

### Q: 为什么不用 Redux？

- React Query 管理服务端状态
- Redux 管理客户端状态
- 项目目前不需要复杂客户端状态

### Q: 如何实现前后端类型共享？

当前手动同步，未来可以：
- 使用 OpenAPI 生成器
- 使用 tRPC
- 使用 GraphQL

---

## 扩展资源

- [React 文档](https://react.dev)
- [TanStack Query](https://tanstack.com/query)
- [TypeScript 手册](https://www.typescriptlang.org/docs/)
- [Vite 指南](https://vitejs.dev/guide/)
- [Axios 文档](https://axios-http.com/)

---

*本学习指引与代码中的知识点注释配合使用，效果更佳。*
