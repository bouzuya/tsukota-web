/**
 * Format amount as Japanese Yen
 */
export function formatAmount(amount: string): string {
  const num = parseInt(amount, 10);
  if (isNaN(num)) return amount;

  const formatter = new Intl.NumberFormat('ja-JP', {
    style: 'currency',
    currency: 'JPY',
  });

  return formatter.format(num);
}

/**
 * Format amount with sign (+ for positive, - for negative)
 */
export function formatAmountWithSign(amount: string): string {
  const num = parseInt(amount, 10);
  if (isNaN(num)) return amount;

  const sign = num >= 0 ? '+' : '';
  return `${sign}${formatAmount(amount)}`;
}
