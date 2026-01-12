import { useAtom, useAtomValue } from 'jotai';
import { useCallback, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  currentUserAtom,
  authLoadingAtom,
  isAuthenticatedAtom,
  getManualUserId,
  setManualUserId,
} from '../atoms/auth';

export function useAuth() {
  const [currentUser, setCurrentUser] = useAtom(currentUserAtom);
  const [authLoading, setAuthLoading] = useAtom(authLoadingAtom);
  const isAuthenticated = useAtomValue(isAuthenticatedAtom);
  const navigate = useNavigate();

  const checkAuth = useCallback(() => {
    setAuthLoading(true);

    // Check for manual user ID in localStorage (development mode)
    const manualUserId = getManualUserId();
    if (manualUserId) {
      setCurrentUser({
        id: manualUserId,
        email: `${manualUserId}@example.com`,
        displayName: `User ${manualUserId}`,
        createdAt: new Date().toISOString(),
      });
    } else {
      setCurrentUser(null);
    }

    setAuthLoading(false);
  }, [setCurrentUser, setAuthLoading]);

  const logout = useCallback(() => {
    setManualUserId(null);
    setCurrentUser(null);
    navigate('/login');
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
