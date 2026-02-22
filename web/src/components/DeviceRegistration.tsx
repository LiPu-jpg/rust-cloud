import { useState } from 'react';
import { Plus } from 'lucide-react';
import { useRegisterDevice } from '../hooks';

export function DeviceRegistration() {
  const [name, setName] = useState('');
  const registerDevice = useRegisterDevice();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) {
      registerDevice.mutate(name.trim(), {
        onSuccess: () => setName(''),
      });
    }
  };

  return (
    <form onSubmit={handleSubmit} className="bg-white rounded-xl shadow-sm border border-gray-200 p-6">
      <h3 className="text-lg font-semibold mb-4">注册新设备</h3>
      <div className="flex gap-3">
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="设备名称"
          className="flex-1 px-4 py-2 rounded-lg border border-gray-300 bg-white focus:outline-none focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500"
        />
        <button
          type="submit"
          disabled={registerDevice.isPending || !name.trim()}
          className="px-4 py-2 bg-blue-500 text-white rounded-lg font-medium hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors flex items-center gap-2"
        >
          <Plus className="w-4 h-4" />
          {registerDevice.isPending ? '注册中...' : '注册'}
        </button>
      </div>
      {registerDevice.isSuccess && (
        <p className="mt-3 text-sm text-green-500">设备注册成功！</p>
      )}
    </form>
  );
}
