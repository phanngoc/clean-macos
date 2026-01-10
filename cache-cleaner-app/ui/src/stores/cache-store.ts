import { create } from 'zustand'
import type { CacheInfo, LargeCacheEntry, IndexedDbItem, CacheCategory } from '@/types/cache'
import { getCacheCategory } from '@/types/cache'
import { scanCaches, cleanCache, scanLargeCaches, cleanLargeCaches, scanIndexedDb, cleanIndexedDbItems } from '@/lib/tauri'

interface CacheState {
  // Data
  caches: CacheInfo[]
  largeCaches: LargeCacheEntry[]
  indexedDbItems: IndexedDbItem[]
  
  // Selection state
  selectedCaches: Set<string> // paths
  selectedLargeCaches: Set<string> // paths
  selectedIndexedDb: Set<string> // paths
  
  // Loading states
  isScanning: boolean
  isCleaning: boolean
  scanProgress: number
  
  // Actions
  scan: () => Promise<void>
  scanLarge: () => Promise<void>
  scanIndexed: (thresholdMb?: number) => Promise<void>
  
  toggleCacheSelection: (path: string) => void
  toggleLargeCacheSelection: (path: string) => void
  toggleIndexedDbSelection: (path: string) => void
  
  selectAllByCategory: (category: CacheCategory) => void
  deselectAllByCategory: (category: CacheCategory) => void
  selectAllLargeCaches: () => void
  deselectAllLargeCaches: () => void
  selectAllIndexedDb: () => void
  deselectAllIndexedDb: () => void
  
  cleanSelectedCaches: (dryRun?: boolean) => Promise<{ success: boolean; message: string; bytesTotal: number }>
  cleanSelectedLargeCaches: () => Promise<{ success: boolean; message: string; bytesTotal: number }>
  cleanSelectedIndexedDb: (dryRun?: boolean) => Promise<{ success: boolean; message: string; bytesTotal: number }>
  
  // Helpers
  getCachesByCategory: (category: CacheCategory) => CacheInfo[]
  getSelectedBytesByCategory: (category: CacheCategory) => number
  getTotalSelectedBytes: () => number
}

export const useCacheStore = create<CacheState>((set, get) => ({
  // Initial state
  caches: [],
  largeCaches: [],
  indexedDbItems: [],
  selectedCaches: new Set(),
  selectedLargeCaches: new Set(),
  selectedIndexedDb: new Set(),
  isScanning: false,
  isCleaning: false,
  scanProgress: 0,
  
  // Scan actions
  scan: async () => {
    console.log('[Store] scan() called')
    set({ isScanning: true, scanProgress: 0 })
    try {
      console.log('[Store] calling scanCaches()...')
      const caches = await scanCaches()
      console.log('[Store] scanCaches() returned:', caches)
      console.log('[Store] caches length:', caches?.length)
      set({ caches, scanProgress: 100 })
      console.log('[Store] state updated with caches')
    } catch (error) {
      console.error('[Store] scan error:', error)
    } finally {
      set({ isScanning: false })
    }
  },
  
  scanLarge: async () => {
    console.log('[Store] scanLarge() called')
    set({ isScanning: true })
    try {
      const largeCaches = await scanLargeCaches()
      console.log('[Store] scanLargeCaches() returned:', largeCaches?.length, 'items')
      set({ largeCaches })
    } catch (error) {
      console.error('[Store] scanLarge error:', error)
    } finally {
      set({ isScanning: false })
    }
  },
  
  scanIndexed: async (thresholdMb?: number) => {
    console.log('[Store] scanIndexed() called')
    set({ isScanning: true })
    try {
      const indexedDbItems = await scanIndexedDb(thresholdMb)
      console.log('[Store] scanIndexedDb() returned:', indexedDbItems?.length, 'items')
      set({ indexedDbItems })
    } catch (error) {
      console.error('[Store] scanIndexed error:', error)
    } finally {
      set({ isScanning: false })
    }
  },
  
  // Toggle selections
  toggleCacheSelection: (path) => {
    const selected = new Set(get().selectedCaches)
    if (selected.has(path)) {
      selected.delete(path)
    } else {
      selected.add(path)
    }
    set({ selectedCaches: selected })
  },
  
  toggleLargeCacheSelection: (path) => {
    const selected = new Set(get().selectedLargeCaches)
    if (selected.has(path)) {
      selected.delete(path)
    } else {
      selected.add(path)
    }
    set({ selectedLargeCaches: selected })
  },
  
  toggleIndexedDbSelection: (path) => {
    const selected = new Set(get().selectedIndexedDb)
    if (selected.has(path)) {
      selected.delete(path)
    } else {
      selected.add(path)
    }
    set({ selectedIndexedDb: selected })
  },
  
  // Bulk selections
  selectAllByCategory: (category) => {
    const caches = get().caches.filter(c => getCacheCategory(c.cache_type) === category && c.exists)
    const selected = new Set(get().selectedCaches)
    caches.forEach(c => selected.add(c.path))
    set({ selectedCaches: selected })
  },
  
  deselectAllByCategory: (category) => {
    const caches = get().caches.filter(c => getCacheCategory(c.cache_type) === category)
    const selected = new Set(get().selectedCaches)
    caches.forEach(c => selected.delete(c.path))
    set({ selectedCaches: selected })
  },
  
  selectAllLargeCaches: () => {
    const selected = new Set(get().largeCaches.map(c => c.path))
    set({ selectedLargeCaches: selected })
  },
  
  deselectAllLargeCaches: () => {
    set({ selectedLargeCaches: new Set() })
  },
  
  selectAllIndexedDb: () => {
    const selected = new Set(get().indexedDbItems.filter(i => i.over_threshold).map(i => i.path))
    set({ selectedIndexedDb: selected })
  },
  
  deselectAllIndexedDb: () => {
    set({ selectedIndexedDb: new Set() })
  },
  
  // Clean actions
  cleanSelectedCaches: async (dryRun = false) => {
    const { selectedCaches, caches } = get()
    if (selectedCaches.size === 0) {
      return { success: false, message: 'No caches selected', bytesTotal: 0 }
    }
    
    set({ isCleaning: true })
    let bytesTotal = 0
    const messages: string[] = []
    
    try {
      // Group selected caches by type
      const selectedCacheItems = caches.filter(c => selectedCaches.has(c.path))
      const types = [...new Set(selectedCacheItems.map(c => c.cache_type))]
      
      for (const type of types) {
        const result = await cleanCache(type, dryRun)
        bytesTotal += result.bytes_cleaned
        messages.push(result.message)
      }
      
      // Rescan after cleaning
      if (!dryRun) {
        await get().scan()
        set({ selectedCaches: new Set() })
      }
      
      return { success: true, message: messages.join('\n'), bytesTotal }
    } catch (error) {
      return { success: false, message: String(error), bytesTotal }
    } finally {
      set({ isCleaning: false })
    }
  },
  
  cleanSelectedLargeCaches: async () => {
    const { selectedLargeCaches } = get()
    if (selectedLargeCaches.size === 0) {
      return { success: false, message: 'No large caches selected', bytesTotal: 0 }
    }
    
    set({ isCleaning: true })
    try {
      const paths = [...selectedLargeCaches]
      const result = await cleanLargeCaches(paths)
      
      // Rescan after cleaning
      await get().scanLarge()
      set({ selectedLargeCaches: new Set() })
      
      return { success: result.success, message: result.message, bytesTotal: result.bytes_cleaned }
    } catch (error) {
      return { success: false, message: String(error), bytesTotal: 0 }
    } finally {
      set({ isCleaning: false })
    }
  },
  
  cleanSelectedIndexedDb: async (dryRun = false) => {
    const { selectedIndexedDb, indexedDbItems } = get()
    if (selectedIndexedDb.size === 0) {
      return { success: false, message: 'No IndexedDB items selected', bytesTotal: 0 }
    }
    
    set({ isCleaning: true })
    try {
      const items = indexedDbItems.filter(i => selectedIndexedDb.has(i.path))
      const result = await cleanIndexedDbItems(items, dryRun)
      
      // Rescan after cleaning
      if (!dryRun) {
        await get().scanIndexed()
        set({ selectedIndexedDb: new Set() })
      }
      
      return { success: result.success, message: result.message, bytesTotal: result.bytes_cleaned }
    } catch (error) {
      return { success: false, message: String(error), bytesTotal: 0 }
    } finally {
      set({ isCleaning: false })
    }
  },
  
  // Helpers
  getCachesByCategory: (category) => {
    return get().caches.filter(c => getCacheCategory(c.cache_type) === category)
  },
  
  getSelectedBytesByCategory: (category) => {
    const { caches, selectedCaches } = get()
    return caches
      .filter(c => getCacheCategory(c.cache_type) === category && selectedCaches.has(c.path))
      .reduce((sum, c) => sum + c.size, 0)
  },
  
  getTotalSelectedBytes: () => {
    const { caches, largeCaches, indexedDbItems, selectedCaches, selectedLargeCaches, selectedIndexedDb } = get()
    
    const cacheBytes = caches
      .filter(c => selectedCaches.has(c.path))
      .reduce((sum, c) => sum + c.size, 0)
    
    const largeBytes = largeCaches
      .filter(c => selectedLargeCaches.has(c.path))
      .reduce((sum, c) => sum + c.size_bytes, 0)
    
    const indexedBytes = indexedDbItems
      .filter(i => selectedIndexedDb.has(i.path))
      .reduce((sum, i) => sum + i.size, 0)
    
    return cacheBytes + largeBytes + indexedBytes
  },
}))
