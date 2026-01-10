import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import type { IndexedDbItem } from '@/types/cache'
import { Database } from 'lucide-react'

interface IndexedDbItemProps {
  item: IndexedDbItem
  isSelected: boolean
  onToggle: () => void
}

export function IndexedDbItemComponent({ item, isSelected, onToggle }: IndexedDbItemProps) {
  return (
    <div
      className={`group flex items-center gap-3 p-4 rounded-lg border transition-all duration-200 cursor-pointer ${
        isSelected 
          ? 'bg-primary/5 border-primary/30 shadow-sm' 
          : 'bg-background border-border hover:border-primary/20 hover:bg-accent/30'
      }`}
      onClick={onToggle}
    >
      <Checkbox checked={isSelected} onCheckedChange={onToggle} className="shrink-0" />
      <Database className="h-4 w-4 text-muted-foreground flex-shrink-0" />
      <div className="flex-1 min-w-0">
        <div className="font-medium text-sm text-foreground truncate mb-0.5">{item.origin}</div>
        <div className="text-xs text-muted-foreground">
          Profile: {item.profile}
        </div>
      </div>
      <div className={`text-sm font-mono font-semibold shrink-0 px-2.5 py-1 rounded-md ${
        item.over_threshold 
          ? isSelected
            ? 'bg-destructive/10 text-destructive'
            : 'bg-destructive/5 text-destructive group-hover:bg-destructive/10'
          : isSelected
            ? 'bg-primary/10 text-primary'
            : 'bg-muted text-muted-foreground group-hover:bg-muted/80'
      } transition-colors`}>
        {formatBytes(item.size)}
      </div>
    </div>
  )
}
