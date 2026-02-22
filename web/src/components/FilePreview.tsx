import { useEffect, useState } from 'react';
import { X, FileText, Hash, Clock, HardDrive } from 'lucide-react';
import type { FileInfo } from '../types';
import { fileApi } from '../api';
import { formatFileSize } from '../utils';

interface FilePreviewProps {
  file: FileInfo | null;
  onClose: () => void;
}

export function FilePreview({ file, onClose }: FilePreviewProps) {
  const [content, setContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!file) return;
    
    // 只尝试加载小文件的文本内容
    if (file.size > 1024 * 1024) {
      setError('文件过大，无法预览');
      return;
    }

    const loadContent = async () => {
      setLoading(true);
      setError(null);
      try {
        const response = await fileApi.getFile(file.path);
        if (response.success && response.data) {
          // 尝试解析为文本
          setContent(response.data.content || '');
        }
      } catch {
        setError('无法加载文件内容');
      } finally {
        setLoading(false);
      }
    };

    loadContent();
  }, [file]);

  if (!file) return null;

  const isImage = /\.(jpg|jpeg|png|gif|webp|svg)$/i.test(file.name);
  const isText = /\.(txt|md|json|js|ts|jsx|tsx|css|html|xml|yaml|yml)$/i.test(file.name);

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-xl shadow-xl max-w-3xl w-full max-h-[80vh] overflow-hidden">
        {/* 头部 */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
          <div className="flex items-center gap-3">
            <FileText className="w-5 h-5 text-gray-500" />
            <h3 className="font-semibold">{file.name}</h3>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-100 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-gray-500" />
          </button>
        </div>

        {/* 元信息 */}
        <div className="px-6 py-3 bg-gray-50 flex items-center gap-6 text-sm text-gray-600">
          <span className="flex items-center gap-1">
            <HardDrive className="w-4 h-4" />
            {formatFileSize(file.size)}
          </span>
          {file.hash && (
            <span className="flex items-center gap-1">
              <Hash className="w-4 h-4" />
              {file.hash.slice(0, 12)}...
            </span>
          )}
          {file.version && (
            <span className="flex items-center gap-1">
              <Clock className="w-4 h-4" />
              v{file.version}
            </span>
          )}
          {file.modified && (
            <span className="flex items-center gap-1">
              <Clock className="w-4 h-4" />
              {new Date(file.modified).toLocaleString()}
            </span>
          )}
        </div>

        {/* 内容区域 */}
        <div className="p-6 overflow-auto max-h-[50vh]">
          {loading && (
            <div className="text-center py-8 text-gray-500">
              加载中...
            </div>
          )}

          {error && (
            <div className="text-center py-8 text-gray-500">
              {error}
            </div>
          )}

          {!loading && !error && content !== null && (
            isText ? (
              <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm font-mono whitespace-pre-wrap">
                {content}
              </pre>
            ) : isImage ? (
              <div className="text-center">
                <img 
                  src={`data:image/${file.name.split('.').pop()};base64,${btoa(content)}`}
                  alt={file.name}
                  className="max-w-full max-h-96 mx-auto rounded"
                />
              </div>
            ) : (
              <div className="text-center py-8 text-gray-500">
                此文件类型不支持预览
              </div>
            )
          )}
        </div>
      </div>
    </div>
  );
}
