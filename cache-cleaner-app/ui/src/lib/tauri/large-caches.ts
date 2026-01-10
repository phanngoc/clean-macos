import { invoke } from '@tauri-apps/api/core'
import type { LargeCacheEntry, CleanResult } from '@/types/cache'

/**
 * Scan for large cache directories (>1GB)
 */
export async function scanLargeCaches(): Promise<LargeCacheEntry[]> {
  return invoke<LargeCacheEntry[]>('scan_large_caches')
}

/**
 * Clean specific large cache directories
 * @param paths - Array of paths to clean
 */
export async function cleanLargeCaches(paths: string[]): Promise<CleanResult> {
  return invoke<CleanResult>('clean_large_caches', { paths })
}
