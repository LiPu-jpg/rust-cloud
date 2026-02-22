import { useState, useMemo } from 'react';
import { FileList } from '../components/FileList';
import { FileUpload } from '../components/FileUpload';
import { FilePreview } from '../components/FilePreview';
import { Breadcrumbs } from '../components/Breadcrumbs';
import { buildBreadcrumbs } from '../utils';
import { useFiles, useDeleteFile, useUploadFile, useCreateFolder } from '../hooks';
import { RefreshCw, Search, FolderPlus } from 'lucide-react';
import type { FileInfo } from '../types';

export function FileManager() {
  const [currentPath, setCurrentPath] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [showUpload, setShowUpload] = useState(false);
  const [showNewFolder, setShowNewFolder] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [previewFile, setPreviewFile] = useState<FileInfo | null>(null);

  const pathParam = currentPath || undefined;
  const { data: filesData, isLoading, refetch } = useFiles(pathParam);
  const deleteFile = useDeleteFile();
  const uploadFile = useUploadFile();
  const createFolder = useCreateFolder();
  
  // 过滤文件（当前目录 + 搜索）
  const files = useMemo(() => {
    const allFiles = filesData?.data || [];
    if (!searchQuery) return allFiles;
    const query = searchQuery.toLowerCase();
    return allFiles.filter(f => f.name.toLowerCase().includes(query));
  }, [filesData?.data, searchQuery]);

  const breadcrumbs = useMemo(() => buildBreadcrumbs(currentPath), [currentPath]);

  const handleNavigate = (path: string) => {
    setCurrentPath(path);
    setSearchQuery('');
  };

  const handleDelete = (path: string) => {
    if (confirm(`确定要删除 "${path}" 吗？`)) {
      deleteFile.mutate(path, {
        onSuccess: () => refetch(),
      });
    }
  };

  const handleUpload = (path: string, content: string) => {
    const fullPath = currentPath ? `${currentPath}/${path}` : path;
    uploadFile.mutate({ path: fullPath, content }, {
      onSuccess: () => {
        setShowUpload(false);
        refetch();
      },
    });
  };

  const handleCreateFolder = () => {
    if (!newFolderName.trim()) return;
    const fullPath = currentPath ? `${currentPath}/${newFolderName}` : newFolderName;
    createFolder.mutate(fullPath, {
      onSuccess: () => {
        setShowNewFolder(false);
        setNewFolderName('');
        refetch();
      },
    });
  };

  const handlePreview = (file: FileInfo) => {
    setPreviewFile(file);
  };

  const handleDownload = (file: FileInfo) => {
    const fullPath = currentPath ? `${currentPath}/${file.name}` : file.name;
    window.open(`/api/files/${encodeURIComponent(fullPath)}`, '_blank');
  };

  return (
    <div className="space-y-6">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">文件管理</h2>
          <p className="text-gray-500 mt-1">管理您的文件和文件夹</p>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => refetch()}
            className="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-lg font-medium hover:bg-gray-50 transition-colors flex items-center gap-2"
            disabled={isLoading}
          >
            <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
            刷新
          </button>
          <button
            onClick={() => setShowNewFolder(!showNewFolder)}
            className="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-lg font-medium hover:bg-gray-50 transition-colors flex items-center gap-2"
          >
            <FolderPlus className="w-4 h-4" />
            新建文件夹
          </button>
          <button
            onClick={() => setShowUpload(!showUpload)}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg font-medium hover:bg-blue-600 transition-colors"
          >
            {showUpload ? '取消' : '上传文件'}
          </button>
        </div>
      </div>

      {/* 面包屑导航 */}
      {currentPath && (
        <Breadcrumbs items={breadcrumbs} onNavigate={handleNavigate} />
      )}

      {/* 新建文件夹 */}
      {showNewFolder && (
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <h3 className="font-semibold mb-4">新建文件夹</h3>
          <div className="flex gap-3">
            <input
              type="text"
              value={newFolderName}
              onChange={(e) => setNewFolderName(e.target.value)}
              placeholder="文件夹名称"
              className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500"
              onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()}
            />
            <button
              onClick={handleCreateFolder}
              disabled={createFolder.isPending || !newFolderName.trim()}
              className="px-4 py-2 bg-blue-500 text-white rounded-lg font-medium hover:bg-blue-600 disabled:opacity-50 transition-colors"
            >
              {createFolder.isPending ? '创建中...' : '创建'}
            </button>
            <button
              onClick={() => {
                setShowNewFolder(false);
                setNewFolderName('');
              }}
              className="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-lg font-medium hover:bg-gray-50 transition-colors"
            >
              取消
            </button>
          </div>
        </div>
      )}

      {/* 搜索框 */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400" />
        <input
          type="text"
          placeholder="搜索文件..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full pl-10 pr-4 py-2 bg-white border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500"
        />
      </div>

      {/* 上传区域 */}
      {showUpload && (
        <FileUpload 
          onUpload={handleUpload} 
          isUploading={uploadFile.isPending}
        />
      )}

      {/* 文件列表 */}
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-semibold">
            {searchQuery ? `搜索结果` : currentPath || '根目录'}
          </h3>
          <span className="text-sm text-gray-500">
            {files.length} 个项目
          </span>
        </div>
        
        {isLoading ? (
          <div className="text-center py-12 text-gray-500">
            <RefreshCw className="w-6 h-6 mx-auto animate-spin mb-2" />
            加载中...
          </div>
        ) : (
          <FileList 
            files={files} 
            onDelete={handleDelete}
            onPreview={handlePreview}
            onDownload={handleDownload}
          />
        )}
      </div>

      {/* 文件预览弹窗 */}
      <FilePreview 
        file={previewFile} 
        onClose={() => setPreviewFile(null)} 
      />
    </div>
  );
}
