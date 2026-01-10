import { Label } from '@/components/ui/label'
import { useSmartScannerStore } from '@/stores'

export function FilterControls() {
  const { minSizeMb, maxAgeDays, setMinSizeMb, setMaxAgeDays, scan, isScanning } = useSmartScannerStore()

  return (
    <div className="flex flex-wrap items-end gap-4 p-4 bg-muted/50 rounded-lg">
      <div className="space-y-2">
        <Label htmlFor="minSize" className="text-sm">
          Min Size (MB)
        </Label>
        <input
          id="minSize"
          type="number"
          min={1}
          value={minSizeMb}
          onChange={(e) => setMinSizeMb(Math.max(1, parseInt(e.target.value) || 1))}
          className="flex h-9 w-24 rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        />
      </div>
      
      <div className="space-y-2">
        <Label htmlFor="maxAge" className="text-sm">
          Max Age (Days)
        </Label>
        <input
          id="maxAge"
          type="number"
          min={1}
          value={maxAgeDays}
          onChange={(e) => setMaxAgeDays(Math.max(1, parseInt(e.target.value) || 1))}
          className="flex h-9 w-24 rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        />
      </div>
      
      <button
        onClick={scan}
        disabled={isScanning}
        className="h-9 px-4 rounded-md bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isScanning ? 'Scanning...' : 'Apply Filters'}
      </button>
    </div>
  )
}
