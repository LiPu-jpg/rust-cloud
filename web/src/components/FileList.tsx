import { useState, useMemo } from 'react';
import { Folder, File, Trash2, MoreVertical, Eye, Download } from 'lucide-react';
import type { FileInfo } from '../types';
import { formatFileSize } from '../utils';

interface FileTreeItemProps {
  file: FileInfo;
  onDelete: (path: string) => void;
  onPreview: (file: FileInfo) => void;
  onDownload: (file: FileInfo) => void;
}

export function FileTreeItem({ file, onDelete, onPreview, onDownload }: FileTreeItemProps) {
  const [showMenu, setShowMenu] = useState(false);

  const handleClick = () => {
    if (!file.is_dir) {
      onPreview(file);
    }
  };

  return (
    <div className="relative group">
      <div
        onClick={handleClick}
        className={`flex items-center gap-2 py-2 px-3 rounded-lg cursor-pointer transition-colors ${
          file.is_dir ? 'hover:bg-gray-50' : 'hover:bg-gray-50'
        }`}
      >
        {file.is_dir ? (
          <Folder className="w-5 h-5 text-blue-500 flex-shrink-0" />
        ) : (
          <File className="w-5 h-5 text-gray-500 flex-shrink-0" />
        )}

        <span className="flex-1 truncate">{file.name}</span>

        {!file.is_dir && (
          <span className="text-sm text-gray-500 flex-shrink-0">
            {formatFileSize(file.size)}
          </span>
        )}

        <div className="relative">
          <button
            onClick={(e) => {
              e.stopPropagation();
              setShowMenu(!showMenu);
            }}
            className="opacity-0 group-hover:opacity-100 p-1 hover:bg-gray-200 rounded transition-opacity"
          >
            <MoreVertical className="w-4 h-4 text-gray-500" />
          </button>
          
          {showMenu && (
            <div className="absolute right-0 top-8 bg-white border border-gray-200 rounded-lg shadow-lg py-1 z-10 min-w-32">
              {!file.is_dir && (
                <>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onPreview(file);
                      setShowMenu(false);
                    }}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2"
                  >
                    <Eye className="w-4 h-4" />
                    预览
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onDownload(file);
                      setShowMenu(false);
                    }}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2"
                  >
                    <Download className="w-4 h-4" />
                    下载
                  </button>
                  <hr className="my-1" />
                </>
              )}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(file.path);
                  setShowMenu(false);
                }}
                className="w-full px-4 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2 text-red-500"
              >
                <Trash2 className="w-4 h-4" />
                删除
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

interface FileListProps {
  files: FileInfo[];
  onDelete: (path: string) => void;
  onPreview: (file: FileInfo) => void;
  onDownload: (file: FileInfo) => void;
}

export function FileList({ files, onDelete, onPreview, onDownload }: FileListProps) {
  const { dirs, regularFiles } = useMemo(() => {
    const dirs = files.filter(f => f.is_dir).sort((a, b) => a.name.localeCompare(b.name));
    const regularFiles = files.filter(f => !f.is_dir).sort((a, b) => a.name.localeCompare(b.name));
    return { dirs, regularFiles };
  }, [files]);

  if (files.length === 0) {
    return (
      <div className="text-center py-12 text-gray-500">
        <Folder className="w-12 h-12 mx-auto mb-3 text-gray-300" />
        <p>暂无文件</p>
      </div>
    );
  }

  return (
    <div className="divide-y divide-gray-100">
      {dirs.map((file) => (
        <FileTreeItem
          key={file.path}
          file={file}
          onDelete={onDelete}
          onPreview={onPreview}
          onDownload={onDownload}
        />
      ))}
      {regularFiles.map((file) => (
        <FileTreeItem
          key={file.path}
          file={file}
          onDelete={onDelete}
          onPreview={onPreview}
          onDownload={onDownload}
        />
      ))}
    </div>
  );
}
