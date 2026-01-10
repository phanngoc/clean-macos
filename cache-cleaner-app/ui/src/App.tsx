import { ThemeProvider } from '@/components/theme-provider'
import { ThemeToggle } from '@/components/theme-toggle'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { CacheCleanerTab, SmartScannerTab } from '@/components/tabs'
import { StatusMessage } from '@/components/StatusMessage'
import { useUiStore } from '@/stores'
import { Sparkles, Trash2, Zap } from 'lucide-react'

function AppContent() {
  const { activeTab, setActiveTab } = useUiStore()

  return (
    <div className="min-h-screen bg-background relative overflow-hidden">
      {/* Decorative blobs for playful background */}
      <div className="blob blob-1 animate-pulse-soft" />
      <div className="blob blob-2 animate-pulse-soft" style={{ animationDelay: '1s' }} />
      
      {/* Header with gradient */}
      <header className="sticky top-0 z-50 w-full glass border-b border-border/50">
        <div className="container flex h-18 items-center justify-between px-6 py-4">
          <div className="flex items-center gap-4">
            {/* Animated logo */}
            <div className="relative">
              <div className="flex items-center justify-center w-12 h-12 rounded-2xl gradient-bg shadow-glow-primary animate-float">
                <Sparkles className="h-6 w-6 text-white" />
              </div>
              {/* Pulsing ring effect */}
              <div className="absolute inset-0 rounded-2xl gradient-bg opacity-30 animate-ping" style={{ animationDuration: '2s' }} />
            </div>
            <div>
              <h1 className="text-2xl font-bold tracking-tight bg-gradient-to-r from-[hsl(8,80%,60%)] via-[hsl(330,75%,60%)] to-[hsl(35,75%,60%)] bg-clip-text text-transparent">
                Cache Cleaner
              </h1>
              <p className="text-sm text-muted-foreground flex items-center gap-1.5">
                <Zap className="h-3 w-3 text-[hsl(50,75%,65%)]" />
                Keep your Mac clean and blazing fast
              </p>
            </div>
          </div>
          <ThemeToggle />
        </div>
      </header>

      {/* Main Content */}
      <main className="container px-6 py-8 max-w-7xl mx-auto relative z-10">
        <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as typeof activeTab)}>
          <TabsList className="grid w-full max-w-lg grid-cols-2 mb-8 h-14 p-1.5 bg-muted/60 glass rounded-2xl">
            <TabsTrigger 
              value="cache-cleaner" 
              className="flex items-center gap-2.5 rounded-xl h-full text-base font-medium data-[state=active]:bg-white data-[state=active]:shadow-lg data-[state=active]:shadow-primary/10 dark:data-[state=active]:bg-card transition-all duration-300"
            >
              <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-gradient-to-br from-[hsl(8,80%,60%)] to-[hsl(35,75%,60%)] shadow-sm">
                <Trash2 className="h-4 w-4 text-white" />
              </div>
              Cache Cleaner
            </TabsTrigger>
            <TabsTrigger 
              value="smart-scanner" 
              className="flex items-center gap-2.5 rounded-xl h-full text-base font-medium data-[state=active]:bg-white data-[state=active]:shadow-lg data-[state=active]:shadow-primary/10 dark:data-[state=active]:bg-card transition-all duration-300"
            >
              <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-gradient-to-br from-[hsl(280,70%,55%)] to-[hsl(240,70%,55%)] shadow-sm">
                <Sparkles className="h-4 w-4 text-white" />
              </div>
              Smart Scanner
            </TabsTrigger>
          </TabsList>

          <TabsContent value="cache-cleaner" className="mt-0 animate-slide-up">
            <CacheCleanerTab />
          </TabsContent>

          <TabsContent value="smart-scanner" className="mt-0 animate-slide-up">
            <SmartScannerTab />
          </TabsContent>
        </Tabs>
      </main>

      {/* Status Toast */}
      <StatusMessage />
      
      {/* Footer decoration */}
      <div className="fixed bottom-0 left-0 right-0 h-1 gradient-bg opacity-60" />
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
