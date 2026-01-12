import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useAtomValue } from 'jotai';
import {
  getAccount,
  updateAccount,
  deleteAccount,
  getExportUrl,
} from '../api/accounts';
import { getUser } from '../api/users';
import { currentUserAtom } from '../atoms/auth';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Select } from '../components/Select';
import { ConfirmModal } from '../components/Modal';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { useRequireAuth } from '../hooks/useAuth';
import { getYearMonthOptions } from '../utils/date';
import type { Account, User } from '../api/types';

export function AccountSettingsPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { authLoading } = useRequireAuth();
  const currentUser = useAtomValue(currentUserAtom);

  const [account, setAccount] = useState<Account | null>(null);
  const [owners, setOwners] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [accountName, setAccountName] = useState('');
  const [saving, setSaving] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [exportMonth, setExportMonth] = useState('');

  useEffect(() => {
    async function fetchData() {
      if (!id || authLoading) return;

      setLoading(true);
      try {
        const acc = await getAccount(id);
        setAccount(acc);
        setAccountName(acc.name);

        // Fetch owner details
        const ownerPromises = acc.ownerIds.map((ownerId) => getUser(ownerId));
        const ownerData = await Promise.all(ownerPromises);
        setOwners(ownerData);
      } finally {
        setLoading(false);
      }
    }
    fetchData();
  }, [id, authLoading]);

  const handleUpdateName = async () => {
    if (!id || !accountName.trim() || !account) return;

    setSaving(true);
    try {
      await updateAccount({ accountId: id, name: accountName.trim() });
      setAccount({ ...account, name: accountName.trim() });
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!id) return;

    setSaving(true);
    try {
      await deleteAccount({ accountId: id });
      navigate('/');
    } finally {
      setSaving(false);
      setShowDeleteModal(false);
    }
  };

  const handleExport = () => {
    if (!id || !exportMonth) return;

    const [year, month] = exportMonth.split('-').map(Number);
    const url = getExportUrl(id, year, month);
    window.open(url, '_blank');
  };

  if (authLoading || loading) {
    return (
      <Layout>
        <div className="flex items-center justify-center py-12">
          <LoadingSpinner size="lg" />
        </div>
      </Layout>
    );
  }

  if (!account) {
    return (
      <Layout>
        <div className="text-center py-12">
          <p className="text-gray-600">アカウントが見つかりません</p>
        </div>
      </Layout>
    );
  }

  const yearMonthOptions = getYearMonthOptions();

  return (
    <Layout>
      <h1 className="text-2xl font-bold text-gray-900 mb-6">アカウント設定</h1>

      <div className="space-y-6">
        {/* Account Name */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            アカウント名
          </h2>
          <div className="flex gap-3">
            <Input
              value={accountName}
              onChange={(e) => setAccountName(e.target.value)}
              className="flex-1"
            />
            <Button
              onClick={handleUpdateName}
              disabled={saving || !accountName.trim() || accountName === account.name}
            >
              {saving ? '保存中...' : '保存'}
            </Button>
          </div>
        </div>

        {/* Owners */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            オーナー一覧
          </h2>
          <div className="space-y-2">
            {owners.map((owner) => (
              <div
                key={owner.id}
                className="flex items-center justify-between py-2 px-3 bg-gray-50 rounded"
              >
                <div>
                  <p className="font-medium text-gray-900">{owner.displayName}</p>
                  <p className="text-sm text-gray-500">{owner.email}</p>
                </div>
                {owner.id === currentUser?.id && (
                  <span className="text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                    あなた
                  </span>
                )}
              </div>
            ))}
          </div>
          <p className="mt-4 text-sm text-gray-500">
            オーナーの追加・削除は現在準備中です。
          </p>
        </div>

        {/* Export */}
        <div className="bg-white rounded-lg shadow p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">
            データエクスポート
          </h2>
          <div className="flex gap-3">
            <Select
              options={yearMonthOptions}
              value={exportMonth}
              onChange={(e) => setExportMonth(e.target.value)}
              placeholder="月を選択"
              className="flex-1"
            />
            <Button
              variant="secondary"
              onClick={handleExport}
              disabled={!exportMonth}
            >
              JSONエクスポート
            </Button>
          </div>
        </div>

        {/* Delete Account */}
        <div className="bg-white rounded-lg shadow p-6 border-2 border-red-200">
          <h2 className="text-lg font-semibold text-red-600 mb-4">
            危険な操作
          </h2>
          <p className="text-gray-600 mb-4">
            アカウントを削除すると、すべての取引データとカテゴリが削除されます。この操作は取り消せません。
          </p>
          <Button variant="danger" onClick={() => setShowDeleteModal(true)}>
            アカウントを削除
          </Button>
        </div>
      </div>

      <ConfirmModal
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        onConfirm={handleDelete}
        title="アカウントを削除"
        message={`「${account.name}」を削除しますか？すべてのデータが完全に削除され、復元できません。`}
        confirmText="削除する"
        variant="danger"
      />
    </Layout>
  );
}
