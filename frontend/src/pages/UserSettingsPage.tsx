import { useState } from 'react';
import { useAtomValue, useSetAtom } from 'jotai';
import { updateUser } from '../api/users';
import { currentUserAtom } from '../atoms/auth';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { useAuth, useRequireAuth } from '../hooks/useAuth';

export function UserSettingsPage() {
  const { authLoading } = useRequireAuth();
  const { logout } = useAuth();
  const currentUser = useAtomValue(currentUserAtom);
  const setCurrentUser = useSetAtom(currentUserAtom);

  const [displayName, setDisplayName] = useState(currentUser?.displayName ?? '');
  const [saving, setSaving] = useState(false);

  const handleUpdateName = async () => {
    if (!currentUser || !displayName.trim()) return;

    setSaving(true);
    try {
      await updateUser({
        userId: currentUser.id,
        displayName: displayName.trim(),
      });
      setCurrentUser({ ...currentUser, displayName: displayName.trim() });
    } finally {
      setSaving(false);
    }
  };

  if (authLoading || !currentUser) {
    return (
      <Layout>
        <div className="flex items-center justify-center py-12">
          <LoadingSpinner size="lg" />
        </div>
      </Layout>
    );
  }

  return (
    <Layout>
      <h1 className="text-2xl font-bold text-gray-900 mb-6">ユーザー設定</h1>

      <div className="space-y-6 max-w-lg">
        {/* User Info */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            アカウント情報
          </h2>
          <div className="space-y-3">
            <div>
              <label className="block text-sm font-medium text-gray-700">
                メールアドレス
              </label>
              <p className="mt-1 text-gray-900">{currentUser.email}</p>
            </div>
          </div>
        </div>

        {/* Display Name */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">表示名</h2>
          <div className="flex gap-3">
            <Input
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              className="flex-1"
            />
            <Button
              onClick={handleUpdateName}
              disabled={
                saving ||
                !displayName.trim() ||
                displayName === currentUser.displayName
              }
            >
              {saving ? '保存中...' : '保存'}
            </Button>
          </div>
        </div>

        {/* Logout */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            ログアウト
          </h2>
          <p className="text-gray-600 mb-4">
            ログアウトすると、再度ログインするまでアプリを使用できません。
          </p>
          <Button variant="secondary" onClick={logout}>
            ログアウト
          </Button>
        </div>
      </div>
    </Layout>
  );
}
