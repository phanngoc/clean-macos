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
  Loader2,
  Sparkles
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
          <h2 className="text-3xl font-bold tracking-tight mb-2 flex items-center gap-3">
            <span className="bg-gradient-to-r from-[hsl(8,80%,60%)] via-[hsl(330,75%,60%)] to-[hsl(35,75%,60%)] bg-clip-text text-transparent">
              Cache Cleaner
            </span>
            {totalSelectedBytes > 0 && (
              <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-bold bg-gradient-to-r from-primary to-[hsl(35,75%,60%)] text-white shadow-lg animate-bounce-in">
                {formatBytes(totalSelectedBytes)}
              </span>
            )}
          </h2>
          <p className="text-muted-foreground flex items-center gap-2">
            <Sparkles className="h-4 w-4 text-[hsl(50,75%,65%)]" />
            Select caches to free up space on your Mac
          </p>
        </div>
        <Button 
          onClick={handleScanAll} 
          disabled={isScanning} 
          variant="outline"
          size="lg"
          className="rounded-xl shadow-md hover:shadow-lg transition-all duration-300 hover:scale-[1.02] border-2"
        >
          {isScanning ? (
            <Loader2 className="h-5 w-5 animate-spin mr-2" />
          ) : (
            <RefreshCw className="h-5 w-5 mr-2" />
          )}
          Rescan All
        </Button>
      </div>

      {/* Progress bar during scan */}
      {isScanning && (
        <div className="space-y-4 p-5 rounded-2xl bg-gradient-to-r from-primary/5 to-[hsl(35,75%,60%)]/5 border-2 border-primary/20 shadow-lg animate-bounce-in">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-primary to-[hsl(35,75%,60%)] flex items-center justify-center shadow-lg">
                <Loader2 className="h-5 w-5 text-white animate-spin" />
              </div>
              <div>
                <span className="font-semibold text-foreground">Scanning your system...</span>
                <p className="text-sm text-muted-foreground">Finding cache files and directories</p>
              </div>
            </div>
            <span className="text-2xl font-bold bg-gradient-to-r from-primary to-[hsl(35,75%,60%)] bg-clip-text text-transparent">
              {Math.round(scanProgress)}%
            </span>
          </div>
          <Progress value={scanProgress} className="h-3" />
        </div>
      )}

      {/* Cache sections with staggered animations */}
      <div className="space-y-6">
        {/* Editor Caches */}
        <div className="animate-slide-up stagger-1">
          <CacheSection
            title="Editor Caches"
            icon={<FileCode className="h-6 w-6" />}
            totalItems={editorCaches.filter(c => c.exists).length}
            selectedCount={editorCaches.filter(c => selectedCaches.has(c.path)).length}
            selectedBytes={getSelectedBytesByCategory('editor')}
            onSelectAll={() => selectAllByCategory('editor')}
            onDeselectAll={() => deselectAllByCategory('editor')}
            onClean={() => handleCleanCaches('editor')}
            isCleaning={isCleaning}
            isEmpty={editorCaches.length === 0}
            emptyMessage="No editor caches found"
            categoryColor="editor"
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
        </div>

        {/* Large Caches */}
        <div className="animate-slide-up stagger-2">
          <CacheSection
            title="Large Caches (>1GB)"
            icon={<HardDrive className="h-6 w-6" />}
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
            categoryColor="large"
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
        </div>

        {/* IndexedDB */}
        <div className="animate-slide-up stagger-3">
          <CacheSection
            title="IndexedDB Storage"
            icon={<Database className="h-6 w-6" />}
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
            categoryColor="database"
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
        </div>

        {/* Browser Caches */}
        <div className="animate-slide-up stagger-4">
          <CacheSection
            title="Browser Caches"
            icon={<Globe className="h-6 w-6" />}
            totalItems={browserCaches.filter(c => c.exists).length}
            selectedCount={browserCaches.filter(c => selectedCaches.has(c.path)).length}
            selectedBytes={getSelectedBytesByCategory('browser')}
            onSelectAll={() => selectAllByCategory('browser')}
            onDeselectAll={() => deselectAllByCategory('browser')}
            onClean={() => handleCleanCaches('browser')}
            isCleaning={isCleaning}
            isEmpty={browserCaches.length === 0}
            emptyMessage="No browser caches found"
            categoryColor="browser"
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
        </div>

        {/* Package Managers */}
        <div className="animate-slide-up stagger-5">
          <CacheSection
            title="Package Managers"
            icon={<Package className="h-6 w-6" />}
            totalItems={packageManagerCaches.filter(c => c.exists).length}
            selectedCount={packageManagerCaches.filter(c => selectedCaches.has(c.path)).length}
            selectedBytes={getSelectedBytesByCategory('packageManager')}
            onSelectAll={() => selectAllByCategory('packageManager')}
            onDeselectAll={() => deselectAllByCategory('packageManager')}
            onClean={() => handleCleanCaches('package manager')}
            isCleaning={isCleaning}
            isEmpty={packageManagerCaches.length === 0}
            emptyMessage="No package manager caches found"
            categoryColor="package"
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
        </div>

        {/* Dev Tools */}
        <div className="animate-slide-up stagger-6">
          <CacheSection
            title="Development Tools"
            icon={<Wrench className="h-6 w-6" />}
            totalItems={devToolsCaches.filter(c => c.exists).length}
            selectedCount={devToolsCaches.filter(c => selectedCaches.has(c.path)).length}
            selectedBytes={getSelectedBytesByCategory('devTools')}
            onSelectAll={() => selectAllByCategory('devTools')}
            onDeselectAll={() => deselectAllByCategory('devTools')}
            onClean={() => handleCleanCaches('development tools')}
            isCleaning={isCleaning}
            isEmpty={devToolsCaches.length === 0}
            emptyMessage="No development tool caches found"
            categoryColor="devtools"
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
        </div>

        {/* System Caches */}
        <div className="animate-slide-up" style={{ animationDelay: '350ms' }}>
          <CacheSection
            title="System Caches"
            icon={<Settings className="h-6 w-6" />}
            totalItems={systemCaches.filter(c => c.exists).length}
            selectedCount={systemCaches.filter(c => selectedCaches.has(c.path)).length}
            selectedBytes={getSelectedBytesByCategory('system')}
            onSelectAll={() => selectAllByCategory('system')}
            onDeselectAll={() => deselectAllByCategory('system')}
            onClean={() => handleCleanCaches('system')}
            isCleaning={isCleaning}
            isEmpty={systemCaches.length === 0}
            emptyMessage="No system caches found"
            categoryColor="system"
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
    </div>
  )
}
