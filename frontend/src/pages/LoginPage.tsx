import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAtomValue } from 'jotai';
import { getAuthUrl } from '../api/auth';
import { isAuthenticatedAtom, authLoadingAtom } from '../atoms/auth';
import { Button } from '../components/Button';
import { PageLoader } from '../components/LoadingSpinner';

export function LoginPage() {
  const isAuthenticated = useAtomValue(isAuthenticatedAtom);
  const authLoading = useAtomValue(authLoadingAtom);
  const navigate = useNavigate();

  useEffect(() => {
    if (!authLoading && isAuthenticated) {
      navigate('/');
    }
  }, [authLoading, isAuthenticated, navigate]);

  const handleLogin = () => {
    window.location.href = getAuthUrl();
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
        </div>

        <Button onClick={handleLogin} className="w-full" size="lg">
          Google でログイン
        </Button>

        <p className="mt-6 text-xs text-center text-gray-500">
          ログインすることで、利用規約とプライバシーポリシーに同意したものとみなされます。
        </p>
      </div>
    </div>
  );
}
