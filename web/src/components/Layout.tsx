import { useState } from 'react';
import { Navigation } from './Navigation';
import { FileManager } from '../pages/FileManager';
import { DeviceManager } from '../pages/DeviceManager';
import { VersionManager } from '../pages/VersionManager';
import { SyncManager } from '../pages/SyncManager';

type Page = 'files' | 'devices' | 'versions' | 'sync';

export function Layout() {
  const [currentPage, setCurrentPage] = useState<Page>('files');

  return (
    <div className="flex h-screen bg-gray-50">
      <Navigation currentPage={currentPage} onPageChange={setCurrentPage} />
      
      <main className="flex-1 overflow-auto">
        <div className="max-w-6xl mx-auto p-8">
          {currentPage === 'files' && <FileManager />}
          {currentPage === 'devices' && <DeviceManager />}
          {currentPage === 'sync' && <SyncManager />}
          {currentPage === 'versions' && <VersionManager />}
        </div>
      </main>
    </div>
  );
}
