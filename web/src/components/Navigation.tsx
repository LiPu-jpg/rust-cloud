import { Folder, HardDrive, History, Settings, RefreshCw } from 'lucide-react';

type Page = 'files' | 'devices' | 'versions' | 'sync';

interface NavigationProps {
  currentPage: Page;
  onPageChange: (page: Page) => void;
}

export function Navigation({ currentPage, onPageChange }: NavigationProps) {
  const navItems: { id: Page; label: string; icon: typeof Folder }[] = [
    { id: 'files', label: '文件管理', icon: Folder },
    { id: 'devices', label: '设备管理', icon: HardDrive },
    { id: 'sync', label: '同步管理', icon: RefreshCw },
    { id: 'versions', label: '版本历史', icon: History },
  ];

  return (
    <nav className="w-64 bg-white border-r border-gray-200 min-h-screen p-4">
      <div className="mb-8">
        <h1 className="text-xl font-bold text-blue-500 flex items-center gap-2">
          <Folder className="w-6 h-6" />
          RustCloud
        </h1>
        <p className="text-sm text-gray-500 mt-1">文件同步服务</p>
      </div>

      <div className="space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentPage === item.id;

          return (
            <button
              key={item.id}
              onClick={() => onPageChange(item.id)}
              className={`
                w-full flex items-center gap-3 px-4 py-3 rounded-lg transition-colors
                ${isActive 
                  ? 'bg-blue-50 text-blue-500 font-medium' 
                  : 'text-gray-600 hover:bg-gray-50'
                }
              `}
            >
              <Icon className="w-5 h-5" />
              {item.label}
            </button>
          );
        })}
      </div>

      <div className="mt-auto pt-8 border-t border-gray-200">
        <button className="w-full flex items-center gap-3 px-4 py-3 text-gray-600 hover:bg-gray-50 rounded-lg transition-colors">
          <Settings className="w-5 h-5" />
          设置
        </button>
      </div>
    </nav>
  );
}
