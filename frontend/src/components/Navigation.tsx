import { Link, useLocation, useParams } from 'react-router-dom';

interface NavItem {
  to: string;
  label: string;
}

export function Navigation() {
  const { id } = useParams<{ id: string }>();
  const location = useLocation();

  if (!id) return null;

  const navItems: NavItem[] = [
    { to: `/accounts/${id}`, label: '収支一覧' },
    { to: `/accounts/${id}/categories`, label: 'カテゴリ' },
    { to: `/accounts/${id}/settings`, label: '設定' },
  ];

  const isActive = (path: string) => {
    if (path === `/accounts/${id}`) {
      return (
        location.pathname === path ||
        location.pathname.startsWith(`/accounts/${id}/new`) ||
        location.pathname.startsWith(`/accounts/${id}/edit`)
      );
    }
    return location.pathname === path;
  };

  return (
    <nav className="bg-gray-50 border-b border-gray-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex gap-4">
          {navItems.map((item) => (
            <Link
              key={item.to}
              to={item.to}
              className={`py-3 px-1 text-sm font-medium border-b-2 transition-colors ${
                isActive(item.to)
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-gray-600 hover:text-gray-900 hover:border-gray-300'
              }`}
            >
              {item.label}
            </Link>
          ))}
        </div>
      </div>
    </nav>
  );
}
