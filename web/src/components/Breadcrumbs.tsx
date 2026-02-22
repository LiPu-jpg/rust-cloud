import { ChevronRight, Home } from 'lucide-react';
import type { BreadcrumbItem } from '../types';

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
  onNavigate: (path: string) => void;
}

export function Breadcrumbs({ items, onNavigate }: BreadcrumbsProps) {
  return (
    <nav className="flex items-center gap-1 text-sm text-gray-600 mb-4">
      <button
        onClick={() => onNavigate('')}
        className="flex items-center gap-1 hover:text-blue-500 transition-colors"
      >
        <Home className="w-4 h-4" />
        根目录
      </button>
      
      {items.map((item, index) => (
        <span key={item.path} className="flex items-center gap-1">
          <ChevronRight className="w-4 h-4 text-gray-400" />
          <button
            onClick={() => onNavigate(item.path)}
            className={`hover:text-blue-500 transition-colors ${
              index === items.length - 1 ? 'text-gray-900 font-medium' : ''
            }`}
          >
            {item.name}
          </button>
        </span>
      ))}
    </nav>
  );
}

export function buildBreadcrumbs(currentPath: string): BreadcrumbItem[] {
  if (!currentPath) return [];
  
  const parts = currentPath.split('/').filter(Boolean);
  const breadcrumbs: BreadcrumbItem[] = [];
  
  let path = '';
  for (const part of parts) {
    path = path ? `${path}/${part}` : part;
    breadcrumbs.push({ name: part, path });
  }
  
  return breadcrumbs;
}
