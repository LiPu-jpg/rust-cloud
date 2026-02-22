import { useState } from 'react';
import { useVersions, useDevices, useSyncPlan, useExecuteSync } from '../hooks';
import { RefreshCw, Upload, Download, Trash2, SkipForward, ArrowRight } from 'lucide-react';
import type { SyncPlanItem } from '../types';

export function SyncManager() {
  const { data: versionsData, isLoading: versionsLoading, refetch: refetchVersions } = useVersions();
  const { data: devicesData } = useDevices();
  const syncPlan = useSyncPlan();
  const executeSync = useExecuteSync();
  
  const [plans, setPlans] = useState<SyncPlanItem[]>([]);
  const [selectedDevice, setSelectedDevice] = useState<string>('');
  const [syncingItems, setSyncingItems] = useState<Set<string>>(new Set());

  const versions = versionsData?.data || [];
  const devices = devicesData?.data || [];

  const handleGeneratePlan = async () => {
    const result = await syncPlan.mutateAsync(versions);
    if (result.success && result.data) {
      setPlans(result.data);
    }
  };

  const handleExecuteSync = async (plan: SyncPlanItem) => {
    if (!selectedDevice) {
      alert('请先选择设备');
      return;
    }
    
    setSyncingItems(prev => new Set(prev).add(plan.file_id));
    
    try {
      await executeSync.mutateAsync({
        fileId: plan.file_id,
        deviceId: selectedDevice,
        action: plan.action,
      });
      setPlans(prev => prev.filter(p => p.file_id !== plan.file_id));
    } catch {
      alert('同步失败');
    } finally {
      setSyncingItems(prev => {
        const next = new Set(prev);
        next.delete(plan.file_id);
        return next;
      });
    }
  };

  const handleExecuteAll = async () => {
    if (!selectedDevice) {
      alert('请先选择设备');
      return;
    }
    
    for (const plan of plans.filter(p => p.action !== 'skip')) {
      await handleExecuteSync(plan);
    }
  };

  const getActionIcon = (action: string) => {
    switch (action) {
      case 'upload': return <Upload className="w-4 h-4 text-blue-500" />;
      case 'download': return <Download className="w-4 h-4 text-green-500" />;
      case 'delete': return <Trash2 className="w-4 h-4 text-red-500" />;
      default: return <SkipForward className="w-4 h-4 text-gray-400" />;
    }
  };

  const getActionLabel = (action: string) => {
    switch (action) {
      case 'upload': return '上传';
      case 'download': return '下载';
      case 'delete': return '删除';
      default: return '跳过';
    }
  };

  const pendingPlans = plans.filter(p => p.action !== 'skip');
  const skipPlans = plans.filter(p => p.action === 'skip');

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">同步管理</h2>
          <p className="text-gray-500 mt-1">管理文件同步计划</p>
        </div>
        <button
          onClick={() => refetchVersions()}
          className="px-4 py-2 bg-white border border-gray-300 text-gray-700 rounded-lg font-medium hover:bg-gray-50 transition-colors flex items-center gap-2"
        >
          <RefreshCw className={`w-4 h-4 ${versionsLoading ? 'animate-spin' : ''}`} />
          刷新
        </button>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h3 className="font-semibold mb-4">1. 选择同步设备</h3>
        <select
          value={selectedDevice}
          onChange={(e) => setSelectedDevice(e.target.value)}
          className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500"
        >
          <option value="">请选择设备</option>
          {devices.map((device) => (
            <option key={device.id} value={device.id}>
              {device.name} ({device.id.slice(0, 8)}...)
            </option>
          ))}
        </select>
      </div>

      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h3 className="font-semibold mb-4">2. 生成同步计划</h3>
        <p className="text-gray-500 text-sm mb-4">
          基于本地文件和服务器版本比较，生成同步计划
        </p>
        <button
          onClick={handleGeneratePlan}
          disabled={syncPlan.isPending}
          className="px-4 py-2 bg-blue-500 text-white rounded-lg font-medium hover:bg-blue-600 disabled:opacity-50 transition-colors"
        >
          {syncPlan.isPending ? '生成中...' : '生成同步计划'}
        </button>
      </div>

      {plans.length > 0 && (
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="font-semibold">3. 同步计划 ({pendingPlans.length} 项待处理)</h3>
            {pendingPlans.length > 0 && (
              <button
                onClick={handleExecuteAll}
                disabled={executeSync.isPending}
                className="px-4 py-2 bg-green-500 text-white rounded-lg font-medium hover:bg-green-600 disabled:opacity-50 transition-colors"
              >
                执行全部同步
              </button>
            )}
          </div>

          {pendingPlans.length > 0 ? (
            <div className="space-y-3">
              {pendingPlans.map((plan) => (
                <div
                  key={plan.file_id}
                  className="flex items-center justify-between p-4 bg-gray-50 rounded-lg"
                >
                  <div className="flex items-center gap-3">
                    <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                      {getActionIcon(plan.action)}
                    </div>
                    <div>
                      <p className="font-medium">{plan.path}</p>
                      <div className="flex items-center gap-2 mt-1 text-sm text-gray-500">
                        <span className={`px-2 py-0.5 rounded-full text-xs ${
                          plan.action === 'upload' ? 'bg-blue-100 text-blue-700' :
                          plan.action === 'download' ? 'bg-green-100 text-green-700' :
                          'bg-red-100 text-red-700'
                        }`}>
                          {getActionLabel(plan.action)}
                        </span>
                      </div>
                    </div>
                  </div>
                  <button
                    onClick={() => handleExecuteSync(plan)}
                    disabled={syncingItems.has(plan.file_id)}
                    className="px-4 py-2 bg-gray-900 text-white rounded-lg font-medium hover:bg-gray-800 disabled:opacity-50 transition-colors flex items-center gap-2"
                  >
                    {syncingItems.has(plan.file_id) ? (
                      '同步中...'
                    ) : (
                      <>
                        执行
                        <ArrowRight className="w-4 h-4" />
                      </>
                    )}
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <div className="text-center py-8 text-gray-500">
              <SkipForward className="w-12 h-12 mx-auto mb-3 text-gray-300" />
              <p>所有文件已是最新，无需同步</p>
            </div>
          )}

          {skipPlans.length > 0 && (
            <div className="mt-6 pt-6 border-t border-gray-200">
              <h4 className="text-sm font-medium text-gray-500 mb-3">已跳过 ({skipPlans.length} 项)</h4>
              <div className="space-y-2">
                {skipPlans.map((plan) => (
                  <div key={plan.file_id} className="flex items-center gap-2 text-sm text-gray-400">
                    <SkipForward className="w-4 h-4" />
                    {plan.path}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
