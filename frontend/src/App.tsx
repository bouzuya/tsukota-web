import { useEffect } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { useAtomValue } from "jotai";
import { useAuth } from "./hooks/useAuth";
import { isAuthenticatedAtom, authLoadingAtom } from "./atoms/auth";
import { PageLoader } from "./components/LoadingSpinner";
import { LoginPage } from "./pages/LoginPage";
import { DashboardPage } from "./pages/DashboardPage";
import { TransactionListPage } from "./pages/TransactionListPage";
import { TransactionFormPage } from "./pages/TransactionFormPage";
import { CategoryManagePage } from "./pages/CategoryManagePage";
import { AccountSettingsPage } from "./pages/AccountSettingsPage";
import { UserSettingsPage } from "./pages/UserSettingsPage";

function AppRoutes() {
	const { checkAuth } = useAuth();
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const authLoading = useAtomValue(authLoadingAtom);

	useEffect(() => {
		checkAuth();
	}, [checkAuth]);

	if (authLoading) {
		return <PageLoader />;
	}

	return (
		<Routes>
			<Route path="/login" element={<LoginPage />} />
			<Route
				path="/"
				element={
					isAuthenticated ? <DashboardPage /> : <Navigate to="/login" replace />
				}
			/>
			<Route path="/accounts/:id" element={<TransactionListPage />} />
			<Route path="/accounts/:id/new" element={<TransactionFormPage />} />
			<Route
				path="/accounts/:id/edit/:txId"
				element={<TransactionFormPage />}
			/>
			<Route path="/accounts/:id/categories" element={<CategoryManagePage />} />
			<Route path="/accounts/:id/settings" element={<AccountSettingsPage />} />
			<Route path="/settings" element={<UserSettingsPage />} />
			<Route path="*" element={<Navigate to="/" replace />} />
		</Routes>
	);
}

function App() {
	return (
		<BrowserRouter>
			<AppRoutes />
		</BrowserRouter>
	);
}

export default App;
