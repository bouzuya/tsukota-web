import { type ReactNode, useEffect } from "react";
import { Button } from "./Button";

interface ModalProps {
	isOpen: boolean;
	onClose: () => void;
	title: string;
	children: ReactNode;
	actions?: ReactNode;
}

export function Modal({
	isOpen,
	onClose,
	title,
	children,
	actions,
}: ModalProps) {
	useEffect(() => {
		if (isOpen) {
			document.body.style.overflow = "hidden";
		} else {
			document.body.style.overflow = "";
		}
		return () => {
			document.body.style.overflow = "";
		};
	}, [isOpen]);

	if (!isOpen) return null;

	return (
		<div className="fixed inset-0 z-50 overflow-y-auto">
			<div className="flex min-h-full items-center justify-center p-4">
				{/* Backdrop */}
				{
					// biome-ignore lint/a11y/noStaticElementInteractions: ignore
					// biome-ignore lint/a11y/useKeyWithClickEvents: ignore
					<div
						className="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
						onClick={onClose}
					/>
				}

				{/* Modal content */}
				<div className="relative bg-white rounded-lg shadow-xl w-full max-w-md p-6">
					<h2 className="text-lg font-semibold text-gray-900 mb-4">{title}</h2>
					<div className="text-gray-600">{children}</div>
					{actions && (
						<div className="mt-6 flex justify-end gap-3">{actions}</div>
					)}
				</div>
			</div>
		</div>
	);
}

interface ConfirmModalProps {
	isOpen: boolean;
	onClose: () => void;
	onConfirm: () => void;
	title: string;
	message: string;
	confirmText?: string;
	cancelText?: string;
	variant?: "primary" | "danger";
}

export function ConfirmModal({
	isOpen,
	onClose,
	onConfirm,
	title,
	message,
	confirmText = "確認",
	cancelText = "キャンセル",
	variant = "primary",
}: ConfirmModalProps) {
	return (
		<Modal
			isOpen={isOpen}
			onClose={onClose}
			title={title}
			actions={
				<>
					<Button variant="secondary" onClick={onClose}>
						{cancelText}
					</Button>
					<Button
						variant={variant === "danger" ? "danger" : "primary"}
						onClick={onConfirm}
					>
						{confirmText}
					</Button>
				</>
			}
		>
			<p>{message}</p>
		</Modal>
	);
}
