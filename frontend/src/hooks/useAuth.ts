import { useAtom, useAtomValue } from 'jotai';
import { useCallback, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { getCurrentUser, logout as apiLogout } from '../api/auth';
import { currentUserAtom, authLoadingAtom, isAuthenticatedAtom } from '../atoms/auth';

export function useAuth() {
  const [currentUser, setCurrentUser] = useAtom(currentUserAtom);
  const [authLoading, setAuthLoading] = useAtom(authLoadingAtom);
  const isAuthenticated = useAtomValue(isAuthenticatedAtom);
  const navigate = useNavigate();

  const checkAuth = useCallback(async () => {
    setAuthLoading(true);
    try {
      const user = await getCurrentUser();
      setCurrentUser(user);
    } catch {
      setCurrentUser(null);
    } finally {
      setAuthLoading(false);
    }
  }, [setCurrentUser, setAuthLoading]);

  const logout = useCallback(async () => {
    try {
      await apiLogout();
    } finally {
      setCurrentUser(null);
      navigate('/login');
    }
  }, [setCurrentUser, navigate]);

  return {
    currentUser,
    isAuthenticated,
    authLoading,
    checkAuth,
    logout,
  };
}

export function useRequireAuth() {
  const isAuthenticated = useAtomValue(isAuthenticatedAtom);
  const authLoading = useAtomValue(authLoadingAtom);
  const navigate = useNavigate();

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      navigate('/login');
    }
  }, [authLoading, isAuthenticated, navigate]);

  return { isAuthenticated, authLoading };
}
