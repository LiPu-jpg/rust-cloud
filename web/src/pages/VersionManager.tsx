import { useMemo, useState } from 'react';
import { useVersions } from '../hooks';
import { FileText, Clock, Hash, RefreshCw } from 'lucide-react';
import type { FileRecord } from '../types';
import { formatFileSize } from '../components/FileList';

export function VersionManager() {
  const { data: versionsData, isLoading, refetch } = useVersions();
  const versions = versionsData?.data || [];
  const [filterPath, setFilterPath] = useState('');

  // 按文件路径分组
  const groupedVersions = useMemo(() => {
    const groups: Record<string, FileRecord[]> = {};
    versions.forEach(v => {
      if (!groups[v.path]) {
        groups[v.path] = [];
      }
      groups[v.path].push(v);
    });
    // 每个文件只显示最新版本
    return Object.entries(groups).map(([path, recs]) => ({
      path,
      latest: recs.sort((a, b) => b.version - a.version)[0],
      count: recs.length,
    }));
  }, [versions]);

  // 过滤
  const filtered = useMemo(() => {
    if (!filterPath) return groupedVersions;
    const q = filterPath.toLowerCase();
    return groupedVersions.filter(g => g.path.toLowerCase().includes(q));
  }, [groupedVersions, filterPath]);

  // 统计
  const stats = useMemo(() => ({
    totalFiles: new Set(versions.map(v => v.path)).size,
    totalVersions: versions.length,
    totalSize: versions.reduce((sum, v) => sum + v.size, 0),
  }), [versions]);

  return (
    <div className="space-y-6">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">版本历史</h2>
          <p className="text-gray-500 mt-1">查看文件版本记录</p>
        </div>
        <button
          onClick={() => refetch()}
          className="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-lg font-medium hover:bg-gray-50 transition-colors flex items-center gap-2"
        >
          <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
          刷新
        </button>
      </div>

      {/* 统计卡片 */}
      <div className="grid grid-cols-3 gap-4">
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-4">
          <div className="text-2xl font-bold text-blue-600">{stats.totalFiles}</div>
          <div className="text-sm text-gray-500">文件总数</div>
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-4">
          <div className="text-2xl font-bold text-green-600">{stats.totalVersions}</div>
          <div className="text-sm text-gray-500">版本总数</div>
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-4">
          <div className="text-2xl font-bold text-purple-600">{formatFileSize(stats.totalSize)}</div>
          <div className="text-sm text-gray-500">总大小</div>
        </div>
      </div>

      {/* 搜索 */}
      <input
        type="text"
        placeholder="搜索文件路径..."
        value={filterPath}
        onChange={(e) => setFilterPath(e.target.value)}
        className="w-full px-4 py-2 bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500"
      />

      {/* 版本列表 */}
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        {isLoading ? (
          <div className="text-center py-12">
            <RefreshCw className="w-6 h-6 mx-auto animate-spin text-gray-400" />
            <p className="text-gray-500 mt-2">加载中...</p>
          </div>
        ) : filtered.length === 0 ? (
          <div className="text-center py-12">
            <FileText className="w-12 h-12 mx-auto mb-3 text-gray-300" />
            <p className="text-gray-500">暂无版本记录</p>
          </div>
        ) : (
          <div className="space-y-3">
            {filtered.map(({ path, latest, count }) => (
              <div
                key={path}
                className="p-4 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors"
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                      <FileText className="w-5 h-5 text-blue-600" />
                    </div>
                    <div>
                      <p className="font-medium">{path}</p>
                      <div className="flex items-center gap-4 mt-1 text-sm text-gray-500">
                        <span className="flex items-center gap-1">
                          <Hash className="w-3 h-3" />
                          v{latest.version}
                        </span>
                        <span className="flex items-center gap-1">
                          <Clock className="w-3 h-3" />
                          {new Date(latest.updated_at).toLocaleString()}
                        </span>
                        <span>
                          {formatFileSize(latest.size)}
                        </span>
                        {count > 1 && (
                          <span className="text-blue-500">
                            {count} 个版本
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                  {latest.hash && (
                    <code className="text-xs text-gray-400 font-mono">
                      {latest.hash.slice(0, 8)}...
                    </code>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
