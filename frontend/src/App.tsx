import { useAtomValue } from "jotai";
import { useEffect } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";
import { authLoadingAtom, isAuthenticatedAtom } from "./atoms/auth";
import { PageLoader } from "./components/LoadingSpinner";
import { useAuth } from "./hooks/useAuth";
import { AccountSettingsPage } from "./pages/AccountSettingsPage";
import { CategoryManagePage } from "./pages/CategoryManagePage";
import { DashboardPage } from "./pages/DashboardPage";
import { LoginPage } from "./pages/LoginPage";
import { MonthlySummaryPage } from "./pages/MonthlySummaryPage";
import { TransactionFormPage } from "./pages/TransactionFormPage";
import { TransactionListPage } from "./pages/TransactionListPage";
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
			<Route
				path="/accounts/:id/stats/monthly"
				element={<MonthlySummaryPage />}
			/>
			<Route path="/accounts/:id/settings" element={<AccountSettingsPage />} />
			<Route path="/settings" element={<UserSettingsPage />} />
			<Route path="*" element={<Navigate to="/" replace />} />
		</Routes>
	);
}

function App() {
	return (
		<BrowserRouter basename={import.meta.env.BASE_URL}>
			<AppRoutes />
		</BrowserRouter>
	);
}

export default App;
