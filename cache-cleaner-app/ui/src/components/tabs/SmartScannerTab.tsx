import { useEffect } from 'react'
import { useSmartScannerStore, useUiStore } from '@/stores'
import { SuggestionItem, FilterControls } from '@/components/smart'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { formatBytes } from '@/lib/utils'
import { confirm } from '@/lib/tauri'
import { Sparkles, Trash2, Loader2 } from 'lucide-react'

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
        <h2 className="text-2xl font-bold tracking-tight mb-1">Smart Scanner</h2>
        <p className="text-sm text-muted-foreground">
          AI-scored suggestions for folders safe to delete
          {selectedBytes > 0 && (
            <span className="ml-2 font-semibold text-foreground">
              • {formatBytes(selectedBytes)} selected
            </span>
          )}
        </p>
      </div>

      {/* Filter Controls */}
      <FilterControls />

      {/* Quick Select Buttons */}
      {suggestions.length > 0 && (
        <div className="flex flex-wrap gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => selectByScoreThreshold(0.7)}
            disabled={highScoreCount === 0}
            className="shadow-sm hover:shadow-md transition-shadow"
          >
            Select High Risk ({highScoreCount})
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => selectByScoreThreshold(0.4)}
            disabled={highScoreCount + mediumScoreCount === 0}
            className="shadow-sm hover:shadow-md transition-shadow"
          >
            Select Medium+ ({highScoreCount + mediumScoreCount})
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={selectAll}
            className="shadow-sm hover:shadow-md transition-shadow"
          >
            Select All ({suggestions.length})
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={deselectAll}
            disabled={selectedPaths.size === 0}
          >
            Clear Selection
          </Button>
        </div>
      )}

      {/* Results Card */}
      <Card className="overflow-hidden">
        <CardHeader className="pb-4">
          <div className="flex items-center justify-between flex-wrap gap-4">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-10 h-10 rounded-lg bg-accent">
                <Sparkles className="h-5 w-5 text-primary" />
              </div>
              <div>
                <CardTitle className="text-lg font-semibold">Suggestions</CardTitle>
                <p className="text-xs text-muted-foreground mt-0.5">
                  {suggestions.length} {suggestions.length === 1 ? 'item' : 'items'} found
                </p>
              </div>
            </div>
            {suggestions.length > 0 && (
              <div className="flex items-center gap-3">
                <div className="flex items-center gap-3 px-3 py-1.5 rounded-lg bg-muted/50">
                  <Checkbox
                    checked={allSelected}
                    onCheckedChange={() => {
                      if (allSelected || someSelected) {
                        deselectAll()
                      } else {
                        selectAll()
                      }
                    }}
                  />
                  {selectedPaths.size > 0 && (
                    <>
                      <span className="text-sm font-medium text-foreground">
                        {selectedPaths.size} selected
                      </span>
                      <span className="text-xs text-muted-foreground font-mono">
                        {formatBytes(selectedBytes)}
                      </span>
                    </>
                  )}
                </div>
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={handleDelete}
                  disabled={selectedPaths.size === 0 || isDeleting}
                  className="shadow-sm hover:shadow-md transition-shadow"
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
            <div className="flex flex-col items-center justify-center py-16 gap-4">
              <Loader2 className="h-10 w-10 animate-spin text-primary" />
              <p className="text-muted-foreground font-medium">Analyzing folders...</p>
            </div>
          ) : suggestions.length === 0 ? (
            <div className="text-center py-12 text-muted-foreground">
              <Sparkles className="h-12 w-12 mx-auto mb-4 opacity-50" />
              <p className="text-sm">No suggestions found with current filters.</p>
              <p className="text-xs mt-2">Try adjusting the minimum size or maximum age.</p>
            </div>
          ) : (
            <div className="space-y-2 max-h-[500px] overflow-y-auto pr-2 -mr-2">
              {suggestions
                .sort((a, b) => b.score - a.score)
                .map((suggestion) => (
                  <SuggestionItem
                    key={suggestion.path}
                    suggestion={suggestion}
                    isSelected={selectedPaths.has(suggestion.path)}
                    onToggle={() => toggleSelection(suggestion.path)}
                  />
                ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
