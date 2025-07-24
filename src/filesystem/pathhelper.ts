export function getFilenameFromPath(filePath: string): string | undefined {
    if (!filePath) return undefined;
    const decoded = decodeURIComponent(filePath);
    const parts = decoded.split(/[\\/]/);
    return parts.length ? parts[parts.length - 1] : undefined;
}