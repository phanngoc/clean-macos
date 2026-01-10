import { useEffect } from 'react'
import { useSmartScannerStore, useUiStore } from '@/stores'
import { SuggestionItem, FilterControls } from '@/components/smart'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import { confirm } from '@/lib/tauri'
import { Sparkles, Trash2, Loader2, Wand2, Zap, Target } from 'lucide-react'

export function SmartScannerTab() {
  const {
    suggestions,
    selectedPaths,
    isScanning,
    isDeleting,
    scan,
    toggleSelection,
    selectAll,
    deselectAll,
    selectByScoreThreshold,
    deleteSelected,
    getSelectedBytes,
  } = useSmartScannerStore()

  const { setStatus } = useUiStore()

  // Initial scan on mount
  useEffect(() => {
    if (suggestions.length === 0) {
      scan()
    }
  }, [])

  const handleDelete = async () => {
    const count = selectedPaths.size
    const bytes = getSelectedBytes()
    
    const confirmed = await confirm(
      'Confirm Deletion',
      `Are you sure you want to delete ${count} folder(s) totaling ${formatBytes(bytes)}?`
    )
    if (!confirmed) return

    const result = await deleteSelected()
    if (result.success) {
      setStatus('success', `Deleted ${formatBytes(result.bytesTotal)}`)
    } else {
      setStatus('error', result.message)
    }
  }

  const allSelected = selectedPaths.size === suggestions.length && suggestions.length > 0
  const someSelected = selectedPaths.size > 0 && selectedPaths.size < suggestions.length
  const selectedBytes = getSelectedBytes()

  const highScoreCount = suggestions.filter(s => s.score >= 0.7).length
  const mediumScoreCount = suggestions.filter(s => s.score >= 0.4 && s.score < 0.7).length

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-3xl font-bold tracking-tight mb-2 flex items-center gap-3">
          <span className="bg-gradient-to-r from-[oklch(0.6_0.2_290)] via-[oklch(0.6_0.2_260)] to-[oklch(0.55_0.18_280)] bg-clip-text text-transparent">
            Smart Scanner
          </span>
          {selectedBytes > 0 && (
            <span className="inline-flex items-center px-3 py-1 rounded-full text-sm font-bold bg-gradient-to-r from-[oklch(0.6_0.2_290)] to-[oklch(0.6_0.2_260)] text-white shadow-lg animate-bounce-in">
              {formatBytes(selectedBytes)}
            </span>
          )}
        </h2>
        <p className="text-muted-foreground flex items-center gap-2">
          <Wand2 className="h-4 w-4 text-[oklch(0.6_0.2_290)]" />
          AI-scored suggestions for folders safe to delete
        </p>
      </div>

      {/* Filter Controls */}
      <FilterControls />

      {/* Quick Select Buttons */}
      {suggestions.length > 0 && (
        <div className="flex flex-wrap gap-3 animate-slide-up">
          <Button
            variant="outline"
            size="default"
            onClick={() => selectByScoreThreshold(0.7)}
            disabled={highScoreCount === 0}
            className="rounded-xl border-2 border-[oklch(0.6_0.22_15)]/30 hover:bg-[oklch(0.6_0.22_15)]/10 hover:border-[oklch(0.6_0.22_15)]/50"
          >
            <Target className="h-4 w-4 text-[oklch(0.6_0.22_15)]" />
            High Risk ({highScoreCount})
          </Button>
          <Button
            variant="outline"
            size="default"
            onClick={() => selectByScoreThreshold(0.4)}
            disabled={highScoreCount + mediumScoreCount === 0}
            className="rounded-xl border-2 border-[oklch(0.75_0.18_70)]/30 hover:bg-[oklch(0.75_0.18_70)]/10 hover:border-[oklch(0.75_0.18_70)]/50"
          >
            <Zap className="h-4 w-4 text-[oklch(0.75_0.18_70)]" />
            Medium+ ({highScoreCount + mediumScoreCount})
          </Button>
          <Button
            variant="outline"
            size="default"
            onClick={selectAll}
            className="rounded-xl border-2"
          >
            <Sparkles className="h-4 w-4" />
            Select All ({suggestions.length})
          </Button>
          <Button
            variant="ghost"
            size="default"
            onClick={deselectAll}
            disabled={selectedPaths.size === 0}
          >
            Clear Selection
          </Button>
        </div>
      )}

      {/* Results Card */}
      <Card className="overflow-hidden border-2 animate-slide-up shadow-xl shadow-[oklch(0.6_0.2_290)]/10">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between flex-wrap gap-4">
            <div className="flex items-center gap-4">
              <div className="relative flex items-center justify-center w-12 h-12 rounded-2xl bg-gradient-to-br from-[oklch(0.6_0.2_290)] to-[oklch(0.55_0.18_310)] shadow-lg">
                <Sparkles className="h-6 w-6 text-white" />
                <div className="absolute inset-0 rounded-2xl bg-gradient-to-tr from-white/20 to-transparent" />
              </div>
              <div>
                <CardTitle className="text-xl font-bold">Suggestions</CardTitle>
                <p className="text-sm text-muted-foreground mt-1 flex items-center gap-1.5">
                  <span className="inline-flex items-center justify-center w-5 h-5 rounded-full text-xs font-bold bg-[oklch(0.6_0.2_290)]/10 text-foreground">
                    {suggestions.length}
                  </span>
                  {suggestions.length === 1 ? 'item' : 'items'} found
                </p>
              </div>
            </div>
            {suggestions.length > 0 && (
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-3 px-4 py-2.5 rounded-xl bg-[oklch(0.6_0.2_290)]/10 border border-border/30">
                  <Checkbox
                    checked={allSelected}
                    onCheckedChange={() => {
                      if (allSelected || someSelected) {
                        deselectAll()
                      } else {
                        selectAll()
                      }
                    }}
                    className="w-5 h-5 rounded-md border-2"
                  />
                  {selectedPaths.size > 0 ? (
                    <div className="flex flex-col">
                      <span className="text-sm font-bold text-foreground">
                        {selectedPaths.size} selected
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
                <Button
                  variant={selectedPaths.size > 0 ? "default" : "outline"}
                  size="lg"
                  onClick={handleDelete}
                  disabled={selectedPaths.size === 0 || isDeleting}
                  className={`rounded-xl font-semibold ${
                    selectedPaths.size > 0 
                      ? 'bg-gradient-to-r from-[oklch(0.6_0.2_290)] to-[oklch(0.55_0.18_310)] hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl hover:scale-[1.02]' 
                      : ''
                  }`}
                >
                  {isDeleting ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Trash2 className="h-4 w-4" />
                  )}
                  Delete Selected
                </Button>
              </div>
            )}
          </div>
        </CardHeader>
        <CardContent className="pt-0">
          {isScanning ? (
            <div className="flex flex-col items-center justify-center py-20 gap-4">
              <div className="relative">
                <div className="w-16 h-16 rounded-2xl bg-gradient-to-br from-[oklch(0.6_0.2_290)] to-[oklch(0.55_0.18_310)] flex items-center justify-center shadow-xl">
                  <Loader2 className="h-8 w-8 text-white animate-spin" />
                </div>
                <div className="absolute inset-0 rounded-2xl bg-gradient-to-br from-[oklch(0.6_0.2_290)] to-[oklch(0.55_0.18_310)] animate-ping opacity-30" />
              </div>
              <div className="text-center">
                <p className="font-semibold text-foreground">Analyzing folders...</p>
                <p className="text-sm text-muted-foreground">This may take a moment</p>
              </div>
            </div>
          ) : suggestions.length === 0 ? (
            <div className="text-center py-20">
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-[oklch(0.6_0.2_290)]/10 mb-4">
                <Sparkles className="h-8 w-8 text-muted-foreground/50" />
              </div>
              <p className="font-medium text-muted-foreground">No suggestions found with current filters</p>
              <p className="text-sm text-muted-foreground/60 mt-1">Try adjusting the minimum size or maximum age 🔍</p>
            </div>
          ) : (
            <div className="space-y-2 max-h-[500px] overflow-y-auto pr-2 -mr-2 scroll-smooth">
              {suggestions
                .sort((a, b) => b.score - a.score)
                .map((suggestion, index) => (
                  <div 
                    key={suggestion.path} 
                    className="animate-slide-up"
                    style={{ animationDelay: `${index * 30}ms` }}
                  >
                    <SuggestionItem
                      suggestion={suggestion}
                      isSelected={selectedPaths.has(suggestion.path)}
                      onToggle={() => toggleSelection(suggestion.path)}
                    />
                  </div>
                ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
