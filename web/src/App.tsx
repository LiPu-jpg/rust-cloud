// [知识点 #210] React Query Provider 配置
// ----------------------------------------
// 题目：QueryClientProvider 有什么作用？
//
// 讲解：
// QueryClientProvider 是 React Query 的核心：
// - 提供全局的 QueryClient 实例
// - 所有 useQuery 和 useMutation 都需要在它的作用域内
// - 管理全局缓存和配置
//
// 思考：是否可以有多个 Provider？
// ----------------------------------------

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Layout } from './components/Layout';
import './index.css';

// [知识点 #211] QueryClient 全局配置
// ----------------------------------------
// 题目：defaultOptions 有什么作用？
//
// 讲解：
// 全局配置影响所有查询：
// - retry: 1 - 请求失败时重试1次
// - refetchOnWindowFocus: false - 窗口重新获得焦点时不自动刷新
// - staleTime: 数据过期时间
// - cacheTime: 缓存保留时间
//
// 这些是合理的默认配置，避免过度请求
// ----------------------------------------

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
    },
  },
});

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <Layout />
    </QueryClientProvider>
  );
}

export default App;
