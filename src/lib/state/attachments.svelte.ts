export type PendingAttachment = {
  id: string;
  name: string;
  mime: string;
  dataUrl: string;
};

const state = $state({ items: [] as PendingAttachment[] });

export function pending(): PendingAttachment[] {
  return state.items;
}

export function addFiles(files: FileList | File[]): void {
  for (const file of Array.from(files)) void addFile(file);
}

export function removePending(id: string): void {
  state.items = state.items.filter((item) => item.id !== id);
}

export function clearPending(): void {
  state.items = [];
}

export function base64Of(dataUrl: string): string {
  const comma = dataUrl.indexOf(",");
  return comma === -1 ? dataUrl : dataUrl.slice(comma + 1);
}

async function addFile(file: File): Promise<void> {
  const rasterized = file.type === "image/svg+xml" ? await rasterizeSvg(file) : null;
  const dataUrl = rasterized ?? (await readAsDataUrl(file));
  const mime = rasterized ? "image/png" : file.type || "application/octet-stream";

  state.items.push({
    id: crypto.randomUUID(),
    name: file.name,
    mime,
    dataUrl,
  });
}

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

// Vision providers generally don't accept SVG, and some hand the model raw
// source instead of an image, so rasterize it to PNG first.
async function rasterizeSvg(file: File): Promise<string | null> {
  const url = URL.createObjectURL(new Blob([await file.text()], { type: "image/svg+xml" }));
  try {
    const image = await loadImage(url);
    const width = image.naturalWidth || 1024;
    const height = image.naturalHeight || 1024;
    const scale = Math.min(2, 2048 / Math.max(width, height));

    const canvas = document.createElement("canvas");
    canvas.width = Math.max(1, Math.round(width * scale));
    canvas.height = Math.max(1, Math.round(height * scale));
    const context = canvas.getContext("2d");
    if (!context) return null;

    context.drawImage(image, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL("image/png");
  } catch {
    return null;
  } finally {
    URL.revokeObjectURL(url);
  }
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("image load failed"));
    image.src = src;
  });
}
