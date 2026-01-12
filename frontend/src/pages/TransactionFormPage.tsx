import { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useAtom } from 'jotai';
import {
  addTransaction,
  updateTransaction,
  deleteTransaction,
  getTransactions,
} from '../api/transactions';
import { getCategories } from '../api/categories';
import { categoriesAtom, activeCategoriesAtom } from '../atoms/categories';
import { Layout } from '../components/Layout';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Select } from '../components/Select';
import { ConfirmModal } from '../components/Modal';
import { LoadingSpinner } from '../components/LoadingSpinner';
import { useRequireAuth } from '../hooks/useAuth';
import { getTodayString } from '../utils/date';
import { useAtomValue } from 'jotai';

export function TransactionFormPage() {
  const { id, txId } = useParams<{ id: string; txId?: string }>();
  const navigate = useNavigate();
  const { authLoading } = useRequireAuth();

  const [categories, setCategories] = useAtom(categoriesAtom);
  const activeCategories = useAtomValue(activeCategoriesAtom);

  const [date, setDate] = useState(getTodayString());
  const [amount, setAmount] = useState('');
  const [categoryId, setCategoryId] = useState('');
  const [comment, setComment] = useState('');

  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);

  const isEditMode = !!txId;

  useEffect(() => {
    async function fetchData() {
      if (!id || authLoading) return;

      setLoading(true);
      try {
        const cats = await getCategories(id);
        setCategories(cats);

        if (txId) {
          // Fetch transaction for editing
          const { items } = await getTransactions(id);
          const tx = items.find((t) => t.id === txId);
          if (tx) {
            setDate(tx.date);
            setAmount(tx.amount);
            setCategoryId(tx.categoryId);
            setComment(tx.comment);
          }
        }
      } finally {
        setLoading(false);
      }
    }
    fetchData();
  }, [id, txId, authLoading, setCategories]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!id || !categoryId || !amount) return;

    setSaving(true);
    try {
      if (isEditMode && txId) {
        await updateTransaction({
          account_id: id,
          transaction_id: txId,
          amount,
          category_id: categoryId,
          comment,
          date,
        });
      } else {
        await addTransaction({
          account_id: id,
          amount,
          category_id: categoryId,
          comment,
          date,
        });
      }
      navigate(`/accounts/${id}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async () => {
    if (!id || !txId) return;

    setSaving(true);
    try {
      await deleteTransaction({
        account_id: id,
        transaction_id: txId,
      });
      navigate(`/accounts/${id}`);
    } finally {
      setSaving(false);
      setShowDeleteModal(false);
    }
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

  const categoryOptions = activeCategories.map((c) => ({
    value: c.id,
    label: c.name,
  }));

  // For edit mode, include the current category even if deleted
  if (isEditMode && categoryId) {
    const currentCategory = categories.find((c) => c.id === categoryId);
    if (currentCategory && !activeCategories.find((c) => c.id === categoryId)) {
      categoryOptions.unshift({
        value: currentCategory.id,
        label: `${currentCategory.name} (削除済み)`,
      });
    }
  }

  return (
    <Layout>
      <div className="max-w-lg mx-auto">
        <h1 className="text-2xl font-bold text-gray-900 mb-6">
          {isEditMode ? '取引を編集' : '取引を追加'}
        </h1>

        <form onSubmit={handleSubmit} className="bg-white rounded-lg shadow p-6">
          <div className="space-y-4">
            <Input
              label="日付"
              type="date"
              value={date}
              onChange={(e) => setDate(e.target.value)}
              required
            />

            <Input
              label="金額"
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="例: -1000 (支出) or 5000 (収入)"
              required
            />

            <Select
              label="カテゴリ"
              value={categoryId}
              onChange={(e) => setCategoryId(e.target.value)}
              options={categoryOptions}
              placeholder="カテゴリを選択"
              required
            />

            <Input
              label="コメント"
              value={comment}
              onChange={(e) => setComment(e.target.value)}
              placeholder="メモ (任意)"
            />
          </div>

          <div className="mt-6 flex gap-3">
            <Button
              type="button"
              variant="secondary"
              onClick={() => navigate(`/accounts/${id}`)}
              disabled={saving}
            >
              キャンセル
            </Button>
            <Button type="submit" disabled={saving || !categoryId || !amount}>
              {saving ? '保存中...' : '保存'}
            </Button>
            {isEditMode && (
              <Button
                type="button"
                variant="danger"
                onClick={() => setShowDeleteModal(true)}
                disabled={saving}
                className="ml-auto"
              >
                削除
              </Button>
            )}
          </div>
        </form>
      </div>

      <ConfirmModal
        isOpen={showDeleteModal}
        onClose={() => setShowDeleteModal(false)}
        onConfirm={handleDelete}
        title="取引を削除"
        message="この取引を削除しますか？この操作は取り消せません。"
        confirmText="削除"
        variant="danger"
      />
    </Layout>
  );
}
