import type { BreadcrumbItem } from '../types';

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

export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}
