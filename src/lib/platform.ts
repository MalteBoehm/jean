/**
 * Platform detection utilities
 *
 * Uses navigator.userAgent instead of deprecated navigator.platform
 */

export function isMac(): boolean {
  return typeof navigator !== 'undefined' && /Mac/i.test(navigator.userAgent)
}

export function isWindows(): boolean {
  return (
    typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent)
  )
}

/**
 * Get the platform-specific file manager name
 * Returns "Finder" on macOS, "Explorer" on Windows, "Files" on Linux
 */
export function getFileManagerName(): string {
  if (isMac()) return 'Finder'
  if (isWindows()) return 'Explorer'
  return 'Files'
}
