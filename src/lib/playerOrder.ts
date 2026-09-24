export function moveSelectedPlayer(
  order: string[],
  selectedIds: string[] | null,
  id: string,
  direction: -1 | 1,
): string[] {
  const selected = order.filter(item => selectedIds === null || selectedIds.includes(item));
  const from = selected.indexOf(id);
  const to = from + direction;
  if (from < 0 || to < 0 || to >= selected.length) return order;
  [selected[from], selected[to]] = [selected[to], selected[from]];
  return [...selected, ...order.filter(item => !selected.includes(item))];
}
