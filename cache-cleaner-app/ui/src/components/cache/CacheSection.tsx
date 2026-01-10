import type { ReactNode } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import { Trash2, Loader2, Sparkles } from 'lucide-react'

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
  categoryColor?: string
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
  categoryColor = 'primary',
}: CacheSectionProps) {
  const allSelected = selectedCount === totalItems && totalItems > 0
  const someSelected = selectedCount > 0 && selectedCount < totalItems

  const colorClasses: Record<string, string> = {
    editor: 'from-[hsl(240,70%,55%)] to-[hsl(280,65%,45%)]',
    browser: 'from-[hsl(155,65%,45%)] to-[hsl(165,60%,45%)]',
    package: 'from-[hsl(35,85%,55%)] to-[hsl(25,70%,50%)]',
    devtools: 'from-[hsl(70,70%,55%)] to-[hsl(60,65%,50%)]',
    system: 'from-[hsl(280,50%,45%)] to-[hsl(260,45%,40%)]',
    large: 'from-[hsl(8,75%,55%)] to-[hsl(15,75%,50%)]',
    database: 'from-[hsl(280,70%,55%)] to-[hsl(310,65%,45%)]',
    primary: 'from-[hsl(8,80%,60%)] to-[hsl(25,75%,55%)]',
  }

  const bgColorClasses: Record<string, string> = {
    editor: 'bg-[hsl(240,70%,55%)]/10',
    browser: 'bg-[hsl(155,65%,45%)]/10',
    package: 'bg-[hsl(35,85%,55%)]/10',
    devtools: 'bg-[hsl(70,70%,55%)]/10',
    system: 'bg-[hsl(280,50%,45%)]/10',
    large: 'bg-[hsl(8,75%,55%)]/10',
    database: 'bg-[hsl(280,70%,55%)]/10',
    primary: 'bg-primary/10',
  }

  const glowClasses: Record<string, string> = {
    editor: 'shadow-[0_8px_30px_-5px_hsl(240,70%,55%,0.2)]',
    browser: 'shadow-[0_8px_30px_-5px_hsl(155,65%,45%,0.2)]',
    package: 'shadow-[0_8px_30px_-5px_hsl(35,85%,55%,0.2)]',
    devtools: 'shadow-[0_8px_30px_-5px_hsl(70,70%,55%,0.2)]',
    system: 'shadow-[0_8px_30px_-5px_hsl(280,50%,45%,0.2)]',
    large: 'shadow-[0_8px_30px_-5px_hsl(8,75%,55%,0.2)]',
    database: 'shadow-[0_8px_30px_-5px_hsl(280,70%,55%,0.2)]',
    primary: 'shadow-glow-primary',
  }

  return (
    <Card className={`overflow-hidden border-2 border-transparent hover:border-border/50 transition-all duration-300 animate-bounce-in ${!isEmpty ? glowClasses[categoryColor] : ''}`}>
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between flex-wrap gap-4">
          <div className="flex items-center gap-4">
            {/* Gradient icon container */}
            <div className={`relative flex items-center justify-center w-12 h-12 rounded-2xl bg-gradient-to-br ${colorClasses[categoryColor]} shadow-lg`}>
              <div className="text-white">
                {icon}
              </div>
              {/* Subtle shine effect */}
              <div className="absolute inset-0 rounded-2xl bg-gradient-to-tr from-white/20 to-transparent" />
            </div>
            <div>
              <CardTitle className="text-xl font-bold">{title}</CardTitle>
              <p className="text-sm text-muted-foreground mt-1 flex items-center gap-1.5">
                <span className={`inline-flex items-center justify-center w-5 h-5 rounded-full text-xs font-bold ${bgColorClasses[categoryColor]} text-foreground`}>
                  {totalItems}
                </span>
                {totalItems === 1 ? 'item' : 'items'} found
              </p>
            </div>
          </div>
          {!isEmpty && (
            <div className="flex items-center gap-3">
              {/* Selection controls */}
              <div className={`flex items-center gap-3 px-4 py-2.5 rounded-xl ${bgColorClasses[categoryColor]} border border-border/30`}>
                <Checkbox
                  checked={allSelected}
                  onCheckedChange={() => {
                    if (allSelected || someSelected) {
                      onDeselectAll()
                    } else {
                      onSelectAll()
                    }
                  }}
                  className={`w-5 h-5 rounded-md border-2 ${someSelected ? 'data-[state=checked]:bg-primary/50' : ''}`}
                />
                {selectedCount > 0 ? (
                  <div className="flex flex-col">
                    <span className="text-sm font-bold text-foreground">
                      {selectedCount} selected
                    </span>
                    <span className="text-xs text-muted-foreground font-mono">
                      {formatBytes(selectedBytes)}
                    </span>
                  </div>
                ) : (
                  <span className="text-sm text-muted-foreground">
                    Select all
                  </span>
                )}
              </div>
              {/* Clean button with gradient */}
              <Button
                variant={selectedCount > 0 ? "default" : "outline"}
                size="lg"
                onClick={onClean}
                disabled={selectedCount === 0 || isCleaning}
                className={`rounded-xl font-semibold transition-all duration-300 ${
                  selectedCount > 0 
                    ? `bg-gradient-to-r ${colorClasses[categoryColor]} hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl hover:scale-[1.02]` 
                    : ''
                }`}
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
          <div className="text-center py-16">
            <div className={`inline-flex items-center justify-center w-16 h-16 rounded-2xl ${bgColorClasses[categoryColor]} mb-4`}>
              <Sparkles className="h-8 w-8 text-muted-foreground/50" />
            </div>
            <p className="text-muted-foreground font-medium">{emptyMessage}</p>
            <p className="text-sm text-muted-foreground/60 mt-1">Looking good! 🎉</p>
          </div>
        ) : (
          <div className="space-y-2 max-h-[450px] overflow-y-auto pr-2 -mr-2 scroll-smooth">
            {children}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
