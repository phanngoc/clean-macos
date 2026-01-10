import { invoke } from '@tauri-apps/api/core'
import type { CacheInfo, CleanResult } from '@/types/cache'

/**
 * Scan all cache locations and return their info
 */
export async function scanCaches(): Promise<CacheInfo[]> {
  return invoke<CacheInfo[]>('scan_caches')
}

/**
 * Clean a specific cache type
 * @param cacheType - The type of cache to clean (e.g., 'vscode', 'npm', etc.)
 * @param dryRun - If true, only simulate the cleaning without actually deleting
 */
export async function cleanCache(cacheType: string, dryRun: boolean = false): Promise<CleanResult> {
  return invoke<CleanResult>('clean_cache', { cacheType, dryRun })
}
