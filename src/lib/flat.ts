export function flatIndex(counts: number[], groupIndex: number, itemIndex: number): number {
  let offset = 0;
  for (let i = 0; i < groupIndex; i += 1) offset += counts[i];
  return offset + itemIndex;
}
