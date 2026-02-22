import { useMemo } from 'react';
import { useDevices, useDeviceHeartbeat } from '../hooks';
import { DeviceRegistration } from '../components/DeviceRegistration';
import { HardDrive, Clock, Wifi, WifiOff, RefreshCw, Activity } from 'lucide-react';

export function DeviceManager() {
  const { data: devicesData, isLoading, refetch } = useDevices();
  const heartbeat = useDeviceHeartbeat();
  const devices = devicesData?.data || [];

  // 计算设备在线状态（5分钟内活跃视为在线）
  const getDeviceStatus = (lastSeen: string) => {
    const last = new Date(lastSeen).getTime();
    const now = Date.now();
    const diff = now - last;
    return diff < 5 * 60 * 1000; // 5分钟
  };

  // 统计
  const stats = useMemo(() => {
    const online = devices.filter(d => getDeviceStatus(d.last_seen)).length;
    return { total: devices.length, online, offline: devices.length - online };
  }, [devices]);

  const handleHeartbeat = (id: string) => {
    heartbeat.mutate(id);
  };

  return (
    <div className="space-y-6">
      {/* 头部 */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">设备管理</h2>
          <p className="text-gray-500 mt-1">管理同步设备</p>
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
          <div className="flex items-center gap-2 text-2xl font-bold text-gray-900">
            <HardDrive className="w-6 h-6 text-gray-400" />
            {stats.total}
          </div>
          <div className="text-sm text-gray-500">设备总数</div>
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-4">
          <div className="flex items-center gap-2 text-2xl font-bold text-green-600">
            <Wifi className="w-6 h-6" />
            {stats.online}
          </div>
          <div className="text-sm text-gray-500">在线设备</div>
        </div>
        <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-4">
          <div className="flex items-center gap-2 text-2xl font-bold text-gray-400">
            <WifiOff className="w-6 h-6" />
            {stats.offline}
          </div>
          <div className="text-sm text-gray-500">离线设备</div>
        </div>
      </div>

      {/* 注册设备 */}
      <DeviceRegistration />

      {/* 设备列表 */}
      <div className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
        <h3 className="font-semibold mb-4">已注册设备</h3>
        
        {isLoading ? (
          <div className="text-center py-8">
            <RefreshCw className="w-6 h-6 mx-auto animate-spin text-gray-400" />
          </div>
        ) : devices.length === 0 ? (
          <p className="text-gray-500 text-center py-8">暂无设备</p>
        ) : (
          <div className="space-y-3">
            {devices.map((device) => {
              const isOnline = getDeviceStatus(device.last_seen);
              
              return (
                <div
                  key={device.id}
                  className="flex items-center justify-between p-4 bg-gray-50 rounded-lg"
                >
                  <div className="flex items-center gap-3">
                    <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${
                      isOnline ? 'bg-green-100' : 'bg-gray-200'
                    }`}>
                      <HardDrive className={`w-5 h-5 ${isOnline ? 'text-green-600' : 'text-gray-500'}`} />
                    </div>
                    <div>
                      <div className="flex items-center gap-2">
                        <p className="font-medium">{device.name}</p>
                        <span className={`px-2 py-0.5 text-xs rounded-full ${
                          isOnline 
                            ? 'bg-green-100 text-green-700' 
                            : 'bg-gray-200 text-gray-600'
                        }`}>
                          {isOnline ? '在线' : '离线'}
                        </span>
                      </div>
                      <p className="text-xs text-gray-400 font-mono">{device.id}</p>
                    </div>
                  </div>
                  
                  <div className="flex items-center gap-4">
                    <div className="text-right">
                      <div className="flex items-center gap-1 text-sm text-gray-500">
                        <Clock className="w-4 h-4" />
                        最后活跃: {new Date(device.last_seen).toLocaleString()}
                      </div>
                    </div>
                    <button
                      onClick={() => handleHeartbeat(device.id)}
                      disabled={heartbeat.isPending}
                      className="p-2 hover:bg-gray-200 rounded-lg transition-colors disabled:opacity-50"
                      title="发送心跳"
                    >
                      <Activity className={`w-4 h-4 text-gray-500 ${
                        heartbeat.isPending ? 'animate-pulse' : ''
                      }`} />
                    </button>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
