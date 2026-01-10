import { useEffect, useMemo } from 'react'
import { useCacheStore, useUiStore } from '@/stores'
import { CacheSection, CacheItem, LargeCacheItem, IndexedDbItemComponent } from '@/components/cache'
import { Button } from '@/components/ui/button'
import { Progress } from '@/components/ui/progress'
import { formatBytes } from '@/lib/utils'
import { getCacheCategory } from '@/types/cache'
import { confirm } from '@/lib/tauri'
import { 
  FileCode, 
  Globe, 
  Package, 
  Wrench, 
  Settings, 
  HardDrive, 
  Database,
  RefreshCw,
  Loader2
} from 'lucide-react'

export function CacheCleanerTab() {
  const {
    caches, // Need to subscribe to caches for reactivity
    largeCaches,
    indexedDbItems,
    selectedCaches,
    selectedLargeCaches,
    selectedIndexedDb,
    isScanning,
    isCleaning,
    scanProgress,
    scan,
    scanLarge,
    scanIndexed,
    toggleCacheSelection,
    toggleLargeCacheSelection,
    toggleIndexedDbSelection,
    selectAllByCategory,
    deselectAllByCategory,
    selectAllLargeCaches,
    deselectAllLargeCaches,
    selectAllIndexedDb,
    deselectAllIndexedDb,
    cleanSelectedCaches,
    cleanSelectedLargeCaches,
    cleanSelectedIndexedDb,
    getSelectedBytesByCategory,
    getTotalSelectedBytes,
  } = useCacheStore()

  const { setStatus } = useUiStore()

  // Initial scan on mount
  useEffect(() => {
    console.log('[CacheCleanerTab] useEffect triggered, calling initialScan...')
    const initialScan = async () => {
      try {
        console.log('[CacheCleanerTab] Starting Promise.all for scans...')
        await Promise.all([scan(), scanLarge(), scanIndexed()])
        console.log('[CacheCleanerTab] All scans completed')
      } catch (error) {
        console.error('[CacheCleanerTab] Scan error:', error)
      }
    }
    initialScan()
  }, [scan, scanLarge, scanIndexed])

  const handleScanAll = async () => {
    await Promise.all([scan(), scanLarge(), scanIndexed()])
    setStatus('success', 'Scan completed')
  }

  const handleCleanCaches = async (category: string) => {
    const confirmed = await confirm(
      'Confirm Cleanup',
      `Are you sure you want to clean the selected ${category} caches?`
    )
    if (!confirmed) return

    const result = await cleanSelectedCaches()
    if (result.success) {
      setStatus('success', `Cleaned ${formatBytes(result.bytesTotal)}`)
    } else {
      setStatus('error', result.message)
    }
  }

  const handleCleanLargeCaches = async () => {
    const confirmed = await confirm(
      'Confirm Cleanup',
      'Are you sure you want to delete the selected large cache directories?'
    )
    if (!confirmed) return

    const result = await cleanSelectedLargeCaches()
    if (result.success) {
      setStatus('success', `Cleaned ${formatBytes(result.bytesTotal)}`)
    } else {
      setStatus('error', result.message)
    }
  }

  const handleCleanIndexedDb = async () => {
    const confirmed = await confirm(
      'Confirm Cleanup',
      'Are you sure you want to delete the selected IndexedDB data?'
    )
    if (!confirmed) return

    const result = await cleanSelectedIndexedDb()
    if (result.success) {
      setStatus('success', `Cleaned ${formatBytes(result.bytesTotal)}`)
    } else {
      setStatus('error', result.message)
    }
  }

  const editorCaches = useMemo(() => 
    caches.filter(c => getCacheCategory(c.cache_type) === 'editor'), [caches])
  const browserCaches = useMemo(() => 
    caches.filter(c => getCacheCategory(c.cache_type) === 'browser'), [caches])
  const packageManagerCaches = useMemo(() => 
    caches.filter(c => getCacheCategory(c.cache_type) === 'packageManager'), [caches])
  const devToolsCaches = useMemo(() => 
    caches.filter(c => getCacheCategory(c.cache_type) === 'devTools'), [caches])
  const systemCaches = useMemo(() => 
    caches.filter(c => getCacheCategory(c.cache_type) === 'system'), [caches])

  // Debug log - check if caches are categorized correctly
  useEffect(() => {
    console.log('[CacheCleanerTab] caches from store:', caches)
    console.log('[CacheCleanerTab] browserCaches:', browserCaches)
    console.log('[CacheCleanerTab] editorCaches:', editorCaches)
    console.log('[CacheCleanerTab] packageManagerCaches:', packageManagerCaches)
    console.log('[CacheCleanerTab] systemCaches:', systemCaches)
  }, [caches, browserCaches, editorCaches, packageManagerCaches, systemCaches])

  const totalSelectedBytes = getTotalSelectedBytes()

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div>
          <h2 className="text-2xl font-bold tracking-tight mb-1">Cache Cleaner</h2>
          <p className="text-sm text-muted-foreground">
            Select caches to clean
            {totalSelectedBytes > 0 && (
              <span className="ml-2 font-semibold text-foreground">
                • {formatBytes(totalSelectedBytes)} selected
              </span>
            )}
          </p>
        </div>
        <Button 
          onClick={handleScanAll} 
          disabled={isScanning} 
          variant="outline"
          className="shadow-sm hover:shadow-md transition-shadow"
        >
          {isScanning ? (
            <Loader2 className="h-4 w-4 animate-spin mr-2" />
          ) : (
            <RefreshCw className="h-4 w-4 mr-2" />
          )}
          Rescan All
        </Button>
      </div>

      {/* Progress bar during scan */}
      {isScanning && (
        <div className="space-y-3 p-4 rounded-lg bg-muted/50 border">
          <div className="flex items-center justify-between text-sm">
            <span className="font-medium">Scanning your system...</span>
            <span className="text-muted-foreground">{Math.round(scanProgress)}%</span>
          </div>
          <Progress value={scanProgress} className="h-2" />
        </div>
      )}

      {/* Cache sections */}
      <div className="space-y-5">
        {/* Editor Caches */}
        <CacheSection
          title="Editor Caches"
          icon={<FileCode className="h-5 w-5 text-blue-500" />}
          totalItems={editorCaches.filter(c => c.exists).length}
          selectedCount={editorCaches.filter(c => selectedCaches.has(c.path)).length}
          selectedBytes={getSelectedBytesByCategory('editor')}
          onSelectAll={() => selectAllByCategory('editor')}
          onDeselectAll={() => deselectAllByCategory('editor')}
          onClean={() => handleCleanCaches('editor')}
          isCleaning={isCleaning}
          isEmpty={editorCaches.length === 0}
          emptyMessage="No editor caches found"
        >
          {editorCaches.filter(c => c.exists).map((cache) => (
            <CacheItem
              key={cache.path}
              cache={cache}
              isSelected={selectedCaches.has(cache.path)}
              onToggle={() => toggleCacheSelection(cache.path)}
            />
          ))}
        </CacheSection>

        {/* Large Caches */}
        <CacheSection
          title="Large Caches (>1GB)"
          icon={<HardDrive className="h-5 w-5 text-red-500" />}
          totalItems={largeCaches.length}
          selectedCount={selectedLargeCaches.size}
          selectedBytes={largeCaches
            .filter(c => selectedLargeCaches.has(c.path))
            .reduce((sum, c) => sum + c.size_bytes, 0)}
          onSelectAll={selectAllLargeCaches}
          onDeselectAll={deselectAllLargeCaches}
          onClean={handleCleanLargeCaches}
          isCleaning={isCleaning}
          isEmpty={largeCaches.length === 0}
          emptyMessage="No large cache directories found"
        >
          {largeCaches.map((cache) => (
            <LargeCacheItem
              key={cache.path}
              cache={cache}
              isSelected={selectedLargeCaches.has(cache.path)}
              onToggle={() => toggleLargeCacheSelection(cache.path)}
            />
          ))}
        </CacheSection>

        {/* IndexedDB */}
        <CacheSection
          title="IndexedDB Storage"
          icon={<Database className="h-5 w-5 text-purple-500" />}
          totalItems={indexedDbItems.filter(i => i.over_threshold).length}
          selectedCount={selectedIndexedDb.size}
          selectedBytes={indexedDbItems
            .filter(i => selectedIndexedDb.has(i.path))
            .reduce((sum, i) => sum + i.size, 0)}
          onSelectAll={selectAllIndexedDb}
          onDeselectAll={deselectAllIndexedDb}
          onClean={handleCleanIndexedDb}
          isCleaning={isCleaning}
          isEmpty={indexedDbItems.filter(i => i.over_threshold).length === 0}
          emptyMessage="No large IndexedDB origins found (threshold: 10MB)"
        >
          {indexedDbItems.filter(i => i.over_threshold).map((item) => (
            <IndexedDbItemComponent
              key={item.path}
              item={item}
              isSelected={selectedIndexedDb.has(item.path)}
              onToggle={() => toggleIndexedDbSelection(item.path)}
            />
          ))}
        </CacheSection>

        {/* Browser Caches */}
        <CacheSection
          title="Browser Caches"
          icon={<Globe className="h-5 w-5 text-green-500" />}
          totalItems={browserCaches.filter(c => c.exists).length}
          selectedCount={browserCaches.filter(c => selectedCaches.has(c.path)).length}
          selectedBytes={getSelectedBytesByCategory('browser')}
          onSelectAll={() => selectAllByCategory('browser')}
          onDeselectAll={() => deselectAllByCategory('browser')}
          onClean={() => handleCleanCaches('browser')}
          isCleaning={isCleaning}
          isEmpty={browserCaches.length === 0}
          emptyMessage="No browser caches found"
        >
          {browserCaches.filter(c => c.exists).map((cache) => (
            <CacheItem
              key={cache.path}
              cache={cache}
              isSelected={selectedCaches.has(cache.path)}
              onToggle={() => toggleCacheSelection(cache.path)}
            />
          ))}
        </CacheSection>

        {/* Package Managers */}
        <CacheSection
          title="Package Managers"
          icon={<Package className="h-5 w-5 text-orange-500" />}
          totalItems={packageManagerCaches.filter(c => c.exists).length}
          selectedCount={packageManagerCaches.filter(c => selectedCaches.has(c.path)).length}
          selectedBytes={getSelectedBytesByCategory('packageManager')}
          onSelectAll={() => selectAllByCategory('packageManager')}
          onDeselectAll={() => deselectAllByCategory('packageManager')}
          onClean={() => handleCleanCaches('package manager')}
          isCleaning={isCleaning}
          isEmpty={packageManagerCaches.length === 0}
          emptyMessage="No package manager caches found"
        >
          {packageManagerCaches.filter(c => c.exists).map((cache) => (
            <CacheItem
              key={cache.path}
              cache={cache}
              isSelected={selectedCaches.has(cache.path)}
              onToggle={() => toggleCacheSelection(cache.path)}
            />
          ))}
        </CacheSection>

        {/* Dev Tools */}
        <CacheSection
          title="Development Tools"
          icon={<Wrench className="h-5 w-5 text-yellow-500" />}
          totalItems={devToolsCaches.filter(c => c.exists).length}
          selectedCount={devToolsCaches.filter(c => selectedCaches.has(c.path)).length}
          selectedBytes={getSelectedBytesByCategory('devTools')}
          onSelectAll={() => selectAllByCategory('devTools')}
          onDeselectAll={() => deselectAllByCategory('devTools')}
          onClean={() => handleCleanCaches('development tools')}
          isCleaning={isCleaning}
          isEmpty={devToolsCaches.length === 0}
          emptyMessage="No development tool caches found"
        >
          {devToolsCaches.filter(c => c.exists).map((cache) => (
            <CacheItem
              key={cache.path}
              cache={cache}
              isSelected={selectedCaches.has(cache.path)}
              onToggle={() => toggleCacheSelection(cache.path)}
            />
          ))}
        </CacheSection>

        {/* System Caches */}
        <CacheSection
          title="System Caches"
          icon={<Settings className="h-5 w-5 text-gray-500" />}
          totalItems={systemCaches.filter(c => c.exists).length}
          selectedCount={systemCaches.filter(c => selectedCaches.has(c.path)).length}
          selectedBytes={getSelectedBytesByCategory('system')}
          onSelectAll={() => selectAllByCategory('system')}
          onDeselectAll={() => deselectAllByCategory('system')}
          onClean={() => handleCleanCaches('system')}
          isCleaning={isCleaning}
          isEmpty={systemCaches.length === 0}
          emptyMessage="No system caches found"
        >
          {systemCaches.filter(c => c.exists).map((cache) => (
            <CacheItem
              key={cache.path}
              cache={cache}
              isSelected={selectedCaches.has(cache.path)}
              onToggle={() => toggleCacheSelection(cache.path)}
            />
          ))}
        </CacheSection>
      </div>
    </div>
  )
}
