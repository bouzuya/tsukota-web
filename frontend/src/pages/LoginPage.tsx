import { useAtomValue } from "jotai";
import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { authLoadingAtom, isAuthenticatedAtom } from "../atoms/auth";
import { Button } from "../components/Button";
import { PageLoader } from "../components/LoadingSpinner";

const SIGNIN_URL = "/lab/tsukota/auth/signin";
const SIGNUP_URL = "/lab/tsukota/auth/signup";

export function LoginPage() {
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);
	const authLoading = useAtomValue(authLoadingAtom);
	const navigate = useNavigate();

	useEffect(() => {
		if (!authLoading && isAuthenticated) {
			navigate("/");
		}
	}, [authLoading, isAuthenticated, navigate]);

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

				<div className="space-y-4">
					<a href={SIGNIN_URL} className="block">
						<Button className="w-full" size="lg">
							Google でサインイン
						</Button>
					</a>

					<a href={SIGNUP_URL} className="block">
						<Button className="w-full" size="lg" variant="secondary">
							Google でサインアップ
						</Button>
					</a>
				</div>
			</div>
		</div>
	);
}
