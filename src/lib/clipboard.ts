// WebKit often leaves `clipboardData.files` empty for screenshots and images
// copied in a browser; those live only in `items`. Prefer items and fall back
// to files so the same image is not added twice.
export function pastedFiles(clipboard: DataTransfer | null): File[] {
  if (!clipboard) return [];
  const files: File[] = [];
  for (const item of clipboard.items) {
    if (item.kind !== "file") continue;
    const file = item.getAsFile();
    if (file) files.push(file);
  }
  return files.length > 0 ? files : Array.from(clipboard.files);
}
