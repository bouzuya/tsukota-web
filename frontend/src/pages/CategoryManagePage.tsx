import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { useAtom } from "jotai";
import {
	getCategories,
	addCategory,
	updateCategory,
	deleteCategory,
} from "../api/categories";
import { categoriesAtom, categoriesLoadingAtom } from "../atoms/categories";
import { Layout } from "../components/Layout";
import { Button } from "../components/Button";
import { Input } from "../components/Input";
import { Modal, ConfirmModal } from "../components/Modal";
import { LoadingSpinner } from "../components/LoadingSpinner";
import { useRequireAuth } from "../hooks/useAuth";
import type { Category } from "../api/types";

export function CategoryManagePage() {
	const { id } = useParams<{ id: string }>();
	const { authLoading } = useRequireAuth();

	const [categories, setCategories] = useAtom(categoriesAtom);
	const [loading, setLoading] = useAtom(categoriesLoadingAtom);

	const [showAddModal, setShowAddModal] = useState(false);
	const [showEditModal, setShowEditModal] = useState(false);
	const [showDeleteModal, setShowDeleteModal] = useState(false);
	const [selectedCategory, setSelectedCategory] = useState<Category | null>(
		null,
	);
	const [categoryName, setCategoryName] = useState("");
	const [saving, setSaving] = useState(false);

	useEffect(() => {
		async function fetchCategories() {
			if (!id || authLoading) return;

			setLoading(true);
			try {
				const cats = await getCategories(id);
				setCategories(cats.items);
			} finally {
				setLoading(false);
			}
		}
		fetchCategories();
	}, [id, authLoading, setCategories, setLoading]);

	const activeCategories = categories.filter((c) => c.deletedAt === null);

	const handleAdd = async () => {
		if (!id || !categoryName.trim()) return;

		setSaving(true);
		try {
			await addCategory({ accountId: id, name: categoryName.trim() });
			const cats = await getCategories(id);
			setCategories(cats.items);
			setShowAddModal(false);
			setCategoryName("");
		} finally {
			setSaving(false);
		}
	};

	const handleEdit = async () => {
		if (!id || !selectedCategory || !categoryName.trim()) return;

		setSaving(true);
		try {
			await updateCategory({
				accountId: id,
				categoryId: selectedCategory.id,
				name: categoryName.trim(),
			});
			const cats = await getCategories(id);
			setCategories(cats.items);
			setShowEditModal(false);
			setSelectedCategory(null);
			setCategoryName("");
		} finally {
			setSaving(false);
		}
	};

	const handleDelete = async () => {
		if (!id || !selectedCategory) return;

		setSaving(true);
		try {
			await deleteCategory({
				accountId: id,
				categoryId: selectedCategory.id,
			});
			const cats = await getCategories(id);
			setCategories(cats.items);
			setShowDeleteModal(false);
			setSelectedCategory(null);
		} finally {
			setSaving(false);
		}
	};

	const openEditModal = (category: Category) => {
		setSelectedCategory(category);
		setCategoryName(category.name);
		setShowEditModal(true);
	};

	const openDeleteModal = (category: Category) => {
		setSelectedCategory(category);
		setShowDeleteModal(true);
	};

	if (authLoading || loading) {
		return (
			<Layout>
				<div className="flex items-center justify-center py-12">
					<LoadingSpinner size="lg" />
				</div>
			</Layout>
		);
	}

	return (
		<Layout>
			<div className="mb-6 flex justify-between items-center">
				<h1 className="text-2xl font-bold text-gray-900">カテゴリ管理</h1>
				<Button onClick={() => setShowAddModal(true)}>追加</Button>
			</div>

			{activeCategories.length === 0 ? (
				<div className="bg-white rounded-lg shadow p-8 text-center">
					<p className="text-gray-600 mb-4">カテゴリがありません</p>
					<Button onClick={() => setShowAddModal(true)}>
						最初のカテゴリを追加
					</Button>
				</div>
			) : (
				<div className="bg-white rounded-lg shadow divide-y">
					{activeCategories.map((category) => (
						<div
							key={category.id}
							className="p-4 flex justify-between items-center"
						>
							<span className="font-medium text-gray-900">{category.name}</span>
							<div className="flex gap-2">
								<Button
									variant="secondary"
									size="sm"
									onClick={() => openEditModal(category)}
								>
									編集
								</Button>
								<Button
									variant="danger"
									size="sm"
									onClick={() => openDeleteModal(category)}
								>
									削除
								</Button>
							</div>
						</div>
					))}
				</div>
			)}

			{/* Add Modal */}
			<Modal
				isOpen={showAddModal}
				onClose={() => {
					setShowAddModal(false);
					setCategoryName("");
				}}
				title="カテゴリを追加"
				actions={
					<>
						<Button
							variant="secondary"
							onClick={() => {
								setShowAddModal(false);
								setCategoryName("");
							}}
							disabled={saving}
						>
							キャンセル
						</Button>
						<Button
							onClick={handleAdd}
							disabled={saving || !categoryName.trim()}
						>
							{saving ? "追加中..." : "追加"}
						</Button>
					</>
				}
			>
				<Input
					label="カテゴリ名"
					value={categoryName}
					onChange={(e) => setCategoryName(e.target.value)}
					placeholder="例: 食費"
					autoFocus
				/>
			</Modal>

			{/* Edit Modal */}
			<Modal
				isOpen={showEditModal}
				onClose={() => {
					setShowEditModal(false);
					setSelectedCategory(null);
					setCategoryName("");
				}}
				title="カテゴリを編集"
				actions={
					<>
						<Button
							variant="secondary"
							onClick={() => {
								setShowEditModal(false);
								setSelectedCategory(null);
								setCategoryName("");
							}}
							disabled={saving}
						>
							キャンセル
						</Button>
						<Button
							onClick={handleEdit}
							disabled={saving || !categoryName.trim()}
						>
							{saving ? "保存中..." : "保存"}
						</Button>
					</>
				}
			>
				<Input
					label="カテゴリ名"
					value={categoryName}
					onChange={(e) => setCategoryName(e.target.value)}
					autoFocus
				/>
			</Modal>

			{/* Delete Modal */}
			<ConfirmModal
				isOpen={showDeleteModal}
				onClose={() => {
					setShowDeleteModal(false);
					setSelectedCategory(null);
				}}
				onConfirm={handleDelete}
				title="カテゴリを削除"
				message={`「${selectedCategory?.name}」を削除しますか？このカテゴリは新規取引で選択できなくなりますが、既存の取引には影響しません。`}
				confirmText="削除"
				variant="danger"
			/>
		</Layout>
	);
}
