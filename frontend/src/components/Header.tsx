import { useAtomValue } from "jotai";
import { Link } from "react-router-dom";
import { currentUserAtom, isAuthenticatedAtom } from "../atoms/auth";

export function Header() {
	const currentUser = useAtomValue(currentUserAtom);
	const isAuthenticated = useAtomValue(isAuthenticatedAtom);

	return (
		<header className="bg-white shadow-sm border-b border-gray-200">
			<div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
				<div className="flex justify-between items-center h-16">
					<Link to="/" className="text-xl font-bold text-gray-900">
						tsukota
					</Link>

					{isAuthenticated && currentUser && (
						<div className="flex items-center gap-4">
							<Link
								to="/settings"
								className="text-sm text-gray-600 hover:text-gray-900 font-mono"
								title={currentUser.id}
							>
								{currentUser.id.slice(0, 8)}...
							</Link>
						</div>
					)}
				</div>
			</div>
		</header>
	);
}
