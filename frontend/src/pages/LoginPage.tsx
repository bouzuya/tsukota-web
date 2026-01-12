import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAtomValue, useSetAtom } from 'jotai';
import {
  isAuthenticatedAtom,
  authLoadingAtom,
  currentUserAtom,
  setManualUserId,
} from '../atoms/auth';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { PageLoader } from '../components/LoadingSpinner';

export function LoginPage() {
  const isAuthenticated = useAtomValue(isAuthenticatedAtom);
  const authLoading = useAtomValue(authLoadingAtom);
  const setCurrentUser = useSetAtom(currentUserAtom);
  const setAuthLoading = useSetAtom(authLoadingAtom);
  const navigate = useNavigate();

  const [userId, setUserId] = useState('');

  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      navigate('/');
    }
  }, [authLoading, isAuthenticated, navigate]);

  const handleLogin = () => {
    if (!userId.trim()) return;

    // Store the manual user ID
    setManualUserId(userId.trim());

    // Create a mock user for development
    setCurrentUser({
      id: userId.trim(),
      email: `${userId.trim()}@example.com`,
      displayName: `User ${userId.trim()}`,
      createdAt: new Date().toISOString(),
    });

    setAuthLoading(false);
    navigate('/');
  };

  if (authLoading) {
    return <PageLoader />;
  }

  return (
    <div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center px-4">
      <div className="bg-white rounded-lg shadow-lg p-8 w-full max-w-md">
        <div className="text-center mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">tsukota</h1>
          <p className="text-gray-600">アカウント・支出管理アプリ</p>
          <p className="text-sm text-yellow-600 mt-2">(開発モード)</p>
        </div>

        <div className="space-y-4">
          <Input
            label="User ID"
            value={userId}
            onChange={(e) => setUserId(e.target.value)}
            placeholder="ユーザーIDを入力"
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                handleLogin();
              }
            }}
          />

          <Button
            onClick={handleLogin}
            className="w-full"
            size="lg"
            disabled={!userId.trim()}
          >
            ログイン
          </Button>
        </div>

        <p className="mt-6 text-xs text-center text-gray-500">
          開発用: X-User-Id ヘッダーに指定される User ID を入力してください
        </p>
      </div>
    </div>
  );
}
