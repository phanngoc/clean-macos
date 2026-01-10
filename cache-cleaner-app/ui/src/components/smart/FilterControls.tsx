import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { useSmartScannerStore } from '@/stores'
import { SlidersHorizontal, RefreshCw, Loader2 } from 'lucide-react'

export function FilterControls() {
  const { minSizeMb, maxAgeDays, setMinSizeMb, setMaxAgeDays, scan, isScanning } = useSmartScannerStore()

  return (
    <div className="flex flex-wrap items-end gap-4 p-5 bg-gradient-to-r from-[oklch(0.6_0.2_290)]/5 to-[oklch(0.6_0.2_260)]/5 rounded-2xl border-2 border-[oklch(0.6_0.2_290)]/20 animate-slide-up">
      <div className="flex items-center gap-3">
        <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-gradient-to-br from-[oklch(0.6_0.2_290)] to-[oklch(0.55_0.18_310)] shadow-md">
          <SlidersHorizontal className="h-5 w-5 text-white" />
        </div>
        <span className="font-semibold text-foreground">Filters</span>
      </div>
      
      <div className="flex-1" />
      
      <div className="space-y-1.5">
        <Label htmlFor="minSize" className="text-sm font-medium text-muted-foreground">
          Min Size (MB)
        </Label>
        <input
          id="minSize"
          type="number"
          min={1}
          value={minSizeMb}
          onChange={(e) => setMinSizeMb(Math.max(1, parseInt(e.target.value) || 1))}
          className="flex h-11 w-28 rounded-xl border-2 border-border bg-background px-4 py-2 text-sm font-medium shadow-sm transition-all duration-300 placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-[oklch(0.6_0.2_290)]/50 focus:border-[oklch(0.6_0.2_290)]/50 hover:border-border/80"
        />
      </div>
      
      <div className="space-y-1.5">
        <Label htmlFor="maxAge" className="text-sm font-medium text-muted-foreground">
          Max Age (Days)
        </Label>
        <input
          id="maxAge"
          type="number"
          min={1}
          value={maxAgeDays}
          onChange={(e) => setMaxAgeDays(Math.max(1, parseInt(e.target.value) || 1))}
          className="flex h-11 w-28 rounded-xl border-2 border-border bg-background px-4 py-2 text-sm font-medium shadow-sm transition-all duration-300 placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-[oklch(0.6_0.2_290)]/50 focus:border-[oklch(0.6_0.2_290)]/50 hover:border-border/80"
        />
      </div>
      
      <Button
        onClick={scan}
        disabled={isScanning}
        className="h-11 rounded-xl bg-gradient-to-r from-[oklch(0.6_0.2_290)] to-[oklch(0.55_0.18_310)] hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl hover:scale-[1.02] transition-all duration-300"
      >
        {isScanning ? (
          <Loader2 className="h-4 w-4 animate-spin mr-2" />
        ) : (
          <RefreshCw className="h-4 w-4 mr-2" />
        )}
        Apply Filters
      </Button>
    </div>
  )
}
