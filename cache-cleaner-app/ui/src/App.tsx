import { ThemeProvider } from '@/components/theme-provider'
import { ThemeToggle } from '@/components/theme-toggle'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { CacheCleanerTab, SmartScannerTab } from '@/components/tabs'
import { StatusMessage } from '@/components/StatusMessage'
import { useUiStore } from '@/stores'
import { Trash2, Sparkles } from 'lucide-react'

function AppContent() {
  const { activeTab, setActiveTab } = useUiStore()

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-50 w-full border-b bg-background/80 backdrop-blur-md supports-[backdrop-filter]:bg-background/60 shadow-sm">
        <div className="container flex h-16 items-center justify-between px-6">
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-10 h-10 rounded-xl bg-primary/10 text-primary">
              <Trash2 className="h-5 w-5" />
            </div>
            <div>
              <h1 className="text-xl font-semibold tracking-tight">Cache Cleaner</h1>
              <p className="text-xs text-muted-foreground">Keep your Mac clean and fast</p>
            </div>
          </div>
          <ThemeToggle />
        </div>
      </header>

      {/* Main Content */}
      <main className="container px-6 py-8 max-w-7xl mx-auto">
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as typeof activeTab)}>
          <TabsList className="grid w-full max-w-md grid-cols-2 mb-8 h-11 bg-muted/50">
            <TabsTrigger 
              value="cache-cleaner" 
              className="flex items-center gap-2 data-[state=active]:bg-background data-[state=active]:shadow-sm transition-all"
            >
              <Trash2 className="h-4 w-4" />
              Cache Cleaner
            </TabsTrigger>
            <TabsTrigger 
              value="smart-scanner" 
              className="flex items-center gap-2 data-[state=active]:bg-background data-[state=active]:shadow-sm transition-all"
            >
              <Sparkles className="h-4 w-4" />
              Smart Scanner
            </TabsTrigger>
          </TabsList>

          <TabsContent value="cache-cleaner" className="mt-0">
            <CacheCleanerTab />
          </TabsContent>

          <TabsContent value="smart-scanner" className="mt-0">
            <SmartScannerTab />
          </TabsContent>
        </Tabs>
      </main>

      {/* Status Toast */}
      <StatusMessage />
    </div>
  )
}

function App() {
  return (
    <ThemeProvider defaultTheme="system">
      <AppContent />
    </ThemeProvider>
  )
}

export default App
