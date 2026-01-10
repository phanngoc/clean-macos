// Types matching Rust structs from src-tauri/src/cache/

export interface CacheInfo {
  cache_type: string
  path: string
  size: number
  exists: boolean
}

export interface LargeCacheEntry {
  name: string
  path: string
  size_bytes: number
}

export interface IndexedDbItem {
  origin: string
  profile: string
  path: string
  size: number
  over_threshold: boolean
}

export interface FolderSuggestion {
  name: string
  path: string
  size_bytes: number
  score: number // 0.0 - 1.0
  reasons: string[]
  last_accessed_days_ago?: number
}

export interface CleanResult {
  success: boolean
  message: string
  bytes_cleaned: number
}

// Cache type categories for grouping
export type CacheCategory = 
  | 'editor'
  | 'browser'
  | 'packageManager'
  | 'devTools'
  | 'system'
  | 'large'
  | 'indexedDb'

export const CACHE_TYPE_CATEGORIES: Record<string, CacheCategory> = {
  // Editor caches
  vscode: 'editor',
  cursor: 'editor',
  
  // Browser caches
  chrome: 'browser',
  safari: 'browser',
  firefox: 'browser',
  arc: 'browser',
  
  // Package managers
  npm: 'packageManager',
  yarn: 'packageManager',
  pnpm: 'packageManager',
  pip: 'packageManager',
  cocoapods: 'packageManager',
  gradle: 'packageManager',
  cargo: 'packageManager',
  
  // Dev tools
  xcode_derived_data: 'devTools',
  xcode_archives: 'devTools',
  xcode_simulators: 'devTools',
  
  // System
  cache_dir: 'system',
  system_caches: 'system',
  user_logs: 'system',
  temp_files: 'system',
  ios_backups: 'system',
}

export function getCacheCategory(cacheType: string): CacheCategory {
  return CACHE_TYPE_CATEGORIES[cacheType] || 'system'
}

export function getCacheCategoryLabel(category: CacheCategory): string {
  const labels: Record<CacheCategory, string> = {
    editor: 'Editor Caches',
    browser: 'Browser Caches',
    packageManager: 'Package Managers',
    devTools: 'Development Tools',
    system: 'System Caches',
    large: 'Large Caches (>1GB)',
    indexedDb: 'IndexedDB Storage',
  }
  return labels[category]
}

export function getCacheCategoryIcon(category: CacheCategory): string {
  const icons: Record<CacheCategory, string> = {
    editor: '📝',
    browser: '🌐',
    packageManager: '📦',
    devTools: '🛠️',
    system: '⚙️',
    large: '💾',
    indexedDb: '🗄️',
  }
  return icons[category]
}
