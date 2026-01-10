import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import type { CacheInfo } from '@/types/cache'

interface CacheItemProps {
  cache: CacheInfo
  isSelected: boolean
  onToggle: () => void
}

export function CacheItem({ cache, isSelected, onToggle }: CacheItemProps) {
  const pathDisplay = cache.path.replace(/^\/Users\/[^/]+/, '~')
  const displayName = cache.cache_type.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
  
  return (
    <div
      className={`group flex items-center gap-3 p-4 rounded-lg border transition-all duration-200 cursor-pointer ${
        isSelected 
          ? 'bg-primary/5 border-primary/30 shadow-sm' 
          : 'bg-background border-border hover:border-primary/20 hover:bg-accent/30'
      } ${!cache.exists ? 'opacity-50' : ''}`}
      onClick={onToggle}
    >
      <Checkbox
        checked={isSelected}
        onCheckedChange={onToggle}
        disabled={!cache.exists}
        className="shrink-0"
      />
      <div className="flex-1 min-w-0">
        <div className="font-medium text-sm text-foreground truncate mb-0.5">
          {displayName}
        </div>
        <div className="text-xs text-muted-foreground truncate" title={cache.path}>
          {pathDisplay}
        </div>
      </div>
      <div className={`text-sm font-mono font-semibold shrink-0 px-2.5 py-1 rounded-md ${
        isSelected 
          ? 'bg-primary/10 text-primary' 
          : 'bg-muted text-muted-foreground group-hover:bg-muted/80'
      } transition-colors`}>
        {cache.exists ? formatBytes(cache.size) : 'Not found'}
      </div>
    </div>
  )
}
