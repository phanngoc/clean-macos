import type { ReactNode } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import { Trash2, Loader2 } from 'lucide-react'

interface CacheSectionProps {
  title: string
  icon: ReactNode
  children: ReactNode
  totalItems: number
  selectedCount: number
  selectedBytes: number
  onSelectAll: () => void
  onDeselectAll: () => void
  onClean: () => void
  isCleaning: boolean
  isEmpty?: boolean
  emptyMessage?: string
}

export function CacheSection({
  title,
  icon,
  children,
  totalItems,
  selectedCount,
  selectedBytes,
  onSelectAll,
  onDeselectAll,
  onClean,
  isCleaning,
  isEmpty = false,
  emptyMessage = 'No items found',
}: CacheSectionProps) {
  const allSelected = selectedCount === totalItems && totalItems > 0
  const someSelected = selectedCount > 0 && selectedCount < totalItems

  return (
    <Card className="overflow-hidden">
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-accent">
              {icon}
            </div>
            <div>
              <CardTitle className="text-lg font-semibold">{title}</CardTitle>
              <p className="text-xs text-muted-foreground mt-0.5">
                {totalItems} {totalItems === 1 ? 'item' : 'items'} found
              </p>
            </div>
          </div>
          {!isEmpty && (
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-3 px-3 py-1.5 rounded-lg bg-muted/50">
                <Checkbox
                  checked={allSelected}
                  onCheckedChange={() => {
                    if (allSelected || someSelected) {
                      onDeselectAll()
                    } else {
                      onSelectAll()
                    }
                  }}
                  className={someSelected ? 'data-[state=checked]:bg-primary/50' : ''}
                />
                {selectedCount > 0 && (
                  <span className="text-sm font-medium text-foreground">
                    {selectedCount} selected
                  </span>
                )}
                {selectedCount > 0 && (
                  <span className="text-xs text-muted-foreground font-mono">
                    {formatBytes(selectedBytes)}
                  </span>
                )}
              </div>
              <Button
                variant="destructive"
                size="sm"
                onClick={onClean}
                disabled={selectedCount === 0 || isCleaning}
                className="shadow-sm hover:shadow-md transition-shadow"
              >
                {isCleaning ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Trash2 className="h-4 w-4" />
                )}
                Clean
              </Button>
            </div>
          )}
        </div>
      </CardHeader>
      <CardContent className="pt-0">
        {isEmpty ? (
          <div className="text-center py-12 text-muted-foreground">
            <p className="text-sm">{emptyMessage}</p>
          </div>
        ) : (
          <div className="space-y-2 max-h-[400px] overflow-y-auto pr-2 -mr-2">
            {children}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
