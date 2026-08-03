/**
 * download-helper — shared browser-download utility.
 *
 * Extracted from three duplicate blob→download patterns in the API layer.
 * Kept deliberately simple: create a temp anchor, click it, and clean up.
 */

/**
 * Trigger a browser file download from an in-memory Blob.
 *
 * Creates a temporary `<a>` element with an object URL, programmatically
 * clicks it to start the download, then removes the element and revokes
 * the URL to avoid memory leaks.
 *
 * @param blob - The Blob (or File) to download.
 * @param filename - The suggested filename for the browser's Save dialog.
 */
export function downloadBlob(blob: Blob, filename: string): void {
  const url = window.URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  window.URL.revokeObjectURL(url);
}
