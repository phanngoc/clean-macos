import { invoke } from '@tauri-apps/api/core'
import type { IndexedDbItem, CleanResult } from '@/types/cache'

/**
 * Scan IndexedDB storage for all browsers
 * @param thresholdMb - Minimum size in MB to include (default: 10)
 */
export async function scanIndexedDb(thresholdMb?: number): Promise<IndexedDbItem[]> {
  return invoke<IndexedDbItem[]>('scan_indexed_db_items', { thresholdMb })
}

/**
 * Clean specific IndexedDB items
 * @param items - Array of IndexedDB items to clean
 * @param dryRun - If true, only simulate the cleaning
 */
export async function cleanIndexedDbItems(
  items: IndexedDbItem[],
  dryRun: boolean = false
): Promise<CleanResult> {
  const paths = items.map(item => item.path)
  return invoke<CleanResult>('clean_indexed_db_items', { paths, dryRun })
}
