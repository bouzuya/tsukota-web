import { type ReactNode } from "react";
import { Header } from "./Header";
import { Navigation } from "./Navigation";
import { useParams } from "react-router-dom";

interface LayoutProps {
	children: ReactNode;
}

export function Layout({ children }: LayoutProps) {
	const { id } = useParams<{ id: string }>();

	return (
		<div className="min-h-screen bg-gray-100">
			<Header />
			{id && <Navigation />}
			<main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
				{children}
			</main>
		</div>
	);
}
