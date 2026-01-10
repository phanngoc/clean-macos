# Cache Cleaner App - Technical Deep Dive

## 🎯 Overview Showcase Diagram

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#3b82f6', 'primaryTextColor': '#fff', 'primaryBorderColor': '#60a5fa', 'lineColor': '#8b5cf6', 'secondaryColor': '#1e293b', 'tertiaryColor': '#0f172a'}}}%%
flowchart TB
    subgraph App["🧹 Cache Cleaner App"]
        direction TB
        
        subgraph FE["🖥️ Frontend Layer"]
            direction LR
            HTML[HTML/CSS/JS UI]
            Tabs[Tab Navigation]
            Lists[Cache Lists]
            Progress[Progress Indicators]
        end
        
        subgraph Bridge["🌉 Tauri Bridge"]
            direction LR
            IPC[IPC Commands]
            Events[Event System]
            Plugins[Shell & Dialog Plugins]
        end
        
        subgraph BE["⚙️ Backend Layer - Rust"]
            direction TB
            
            subgraph Core["Core Engine"]
                Scanner[🔍 Scanner Engine]
                Cleaner[🗑️ Cleaner Engine]
                Registry[📋 Scanner Registry]
            end
            
            subgraph Specialized["Specialized Modules"]
                Browser[🌐 Browser Caches]
                Editor[📝 Editor Caches]
                Dev[🔧 Dev Tools]
                Package[📦 Package Managers]
                Smart[🧠 Smart Suggestions]
            end
            
            subgraph Infra["Infrastructure"]
                FS[📁 File System Utils]
                Perms[🔐 Permissions]
                Async[⚡ Concurrency]
            end
        end
    end
    
    FE --> Bridge
    Bridge --> BE
    Core --> Specialized
    Specialized --> Infra
    
    style App fill:#0f172a,stroke:#3b82f6,stroke-width:2px
    style FE fill:#1e293b,stroke:#60a5fa
    style Bridge fill:#1e293b,stroke:#8b5cf6
    style BE fill:#1e293b,stroke:#22c55e
```

---

## 🔄 Complete User Flow

```mermaid
%%{init: {'theme': 'dark'}}%%
stateDiagram-v2
    [*] --> AppLaunch: User opens app
    
    AppLaunch --> PermissionCheck: Check permissions
    
    PermissionCheck --> FullAccess: Has Full Disk Access
    PermissionCheck --> LimitedAccess: No Full Disk Access
    
    FullAccess --> ScanAll: Scan all caches
    LimitedAccess --> ScanPartial: Scan accessible caches
    
    ScanAll --> DisplayResults: Show cache list
    ScanPartial --> DisplayResults: Show cache list
    
    DisplayResults --> SelectCaches: User selects caches
    DisplayResults --> SmartScan: Switch to Smart Scanner
    
    SmartScan --> AnalyzeFolders: Analyze with AI scoring
    AnalyzeFolders --> ShowSuggestions: Display suggestions
    ShowSuggestions --> SelectSuggestions: User selects items
    
    SelectCaches --> PreviewClean: Dry run mode
    SelectSuggestions --> PreviewClean: Dry run mode
    
    PreviewClean --> ConfirmClean: User confirms
    ConfirmClean --> ExecuteClean: Clean selected items
    
    ExecuteClean --> ShowProgress: Display progress
    ShowProgress --> ShowResult: Display results
    ShowResult --> DisplayResults: Continue using app
    
    ShowResult --> [*]: User closes app
```

---

## 🏛️ Component Interaction Matrix

```mermaid
%%{init: {'theme': 'dark'}}%%
graph TB
    subgraph Commands["Tauri Commands (26 total)"]
        C1[scan_caches]
        C2[clean_cache]
        C3[scan_large_caches]
        C4[scan_npm_caches]
        C5[scan_indexed_db_items]
        C6[scan_smart_suggestions]
        C7[register_custom_scanner]
    end
    
    subgraph Scanners["Scanner Implementations"]
        S1[BrowserCacheScanner]
        S2[DevToolsScanner]
        S3[PackageManagerScanner]
        S4[LargeCacheScanner]
        S5[IndexedDBScanner]
        S6[SmartSuggestionScanner]
        S7[CustomScanner]
    end
    
    subgraph Output["Output Types"]
        O1[CacheInfo]
        O2[CleanResult]
        O3[LargeCacheEntry]
        O4[NpmCacheEntry]
        O5[IndexedDbItem]
        O6[FolderSuggestion]
        O7[ScanResult]
    end
    
    C1 --> S1 & S2 & S3
    C2 --> O2
    C3 --> S4 --> O3
    C4 --> O4
    C5 --> S5 --> O5
    C6 --> S6 --> O6
    C7 --> S7 --> O7
    S1 --> O1
    S2 --> O1
    S3 --> O1
```

---

## 🧠 Smart Suggestions Algorithm Visualization

```mermaid
%%{init: {'theme': 'dark'}}%%
flowchart TB
    subgraph Input["📂 Folder Input"]
        Path[Folder Path]
        Metadata[File System Metadata]
    end
    
    subgraph FeatureExtraction["🔬 Feature Extraction"]
        direction LR
        Size["Size<br/>(MB)"]
        Age["Last Access<br/>(Days)"]
        Type["Location<br/>Type"]
    end
    
    subgraph Scoring["📊 Scoring Algorithm"]
        direction TB
        
        subgraph SizeCalc["Size Score (0-40)"]
            S1[">10GB = 40"]
            S2["5-10GB = 35"]
            S3["1-5GB = 30"]
            S4["0.5-1GB = 20"]
            S5["100-500MB = 10"]
        end
        
        subgraph AgeCalc["Age Score (0-35)"]
            A1[">1 year = 35"]
            A2["6-12mo = 28"]
            A3["3-6mo = 20"]
            A4["1-3mo = 10"]
        end
        
        subgraph LocCalc["Location Score (0-25)"]
            L1["Cache Dir = 25"]
            L2["Log Dir = 20"]
            L3["Build Dir = 15"]
            L4["Dev Dir = 10"]
        end
    end
    
    subgraph Output["🎯 Output"]
        Score["Final Score<br/>(0-100)"]
        Reasons["Reason Tags"]
        Priority["Clean Priority"]
    end
    
    Path --> Size & Age & Type
    Metadata --> Size & Age & Type
    Size --> SizeCalc
    Age --> AgeCalc
    Type --> LocCalc
    SizeCalc --> Score
    AgeCalc --> Score
    LocCalc --> Score
    Score --> Reasons
    Score --> Priority
    
    style Score fill:#ef4444,stroke:#fca5a5
```

---

## 🗂️ Cache Type Coverage

```mermaid
%%{init: {'theme': 'dark'}}%%
pie showData
    title Cache Types Distribution
    "Browser Caches" : 4
    "Editor Caches" : 2
    "Package Managers" : 6
    "Xcode" : 3
    "System Caches" : 4
    "Special (IndexedDB, Large)" : 2
    "Custom Scanners" : 1
```

---

## ⚡ Async Processing Flow

```mermaid
%%{init: {'theme': 'dark'}}%%
sequenceDiagram
    autonumber
    participant UI as 🖥️ Frontend
    participant T as 🌉 Tauri
    participant R as ⚙️ Rust Runtime
    participant S as 🔄 Semaphore
    participant W as 👷 Workers
    participant FS as 📁 File System

    UI->>T: invoke("scan_large_caches")
    T->>R: spawn async task
    R->>S: acquire semaphore (4 permits)
    
    par Worker 1
        S->>W: permit granted
        W->>FS: scan dir 1
        FS-->>W: size info
        W->>S: release permit
    and Worker 2
        S->>W: permit granted
        W->>FS: scan dir 2
        FS-->>W: size info
        W->>S: release permit
    and Worker 3
        S->>W: permit granted
        W->>FS: scan dir 3
        FS-->>W: size info
        W->>S: release permit
    and Worker 4
        S->>W: permit granted
        W->>FS: scan dir 4
        FS-->>W: size info
        W->>S: release permit
    end
    
    R->>R: aggregate results
    R-->>T: Vec<LargeCacheEntry>
    T-->>UI: JSON response
```

---

## 🔐 Security & Permission Model

```mermaid
%%{init: {'theme': 'dark'}}%%
flowchart TB
    subgraph Checks["🔍 Permission Checks"]
        FDA["Full Disk Access<br/>/Library/Preferences/"]
        Home["Home Directory<br/>Access Check"]
        Process["Process Check<br/>(Chrome running?)"]
    end
    
    subgraph Access["📂 Access Levels"]
        Full["✅ Full Access<br/>All caches visible"]
        Limited["⚠️ Limited Access<br/>Some caches hidden"]
        Blocked["❌ Blocked<br/>Browser must close"]
    end
    
    subgraph Protection["🛡️ Protection"]
        DryRun["Dry Run Mode<br/>Preview only"]
        Whitelist["Whitelist<br/>Critical paths protected"]
        Threshold["Size Threshold<br/>Min 100MB for suggestions"]
    end
    
    FDA -->|has| Full
    FDA -->|missing| Limited
    Home -->|accessible| Full
    Process -->|running| Blocked
    Process -->|closed| Full
    
    Full --> DryRun
    Limited --> DryRun
    DryRun --> Whitelist
    Whitelist --> Threshold
    
    style Full fill:#22c55e,stroke:#4ade80
    style Limited fill:#f59e0b,stroke:#fbbf24
    style Blocked fill:#ef4444,stroke:#f87171
```

---

## 📱 UI Component Structure

```mermaid
%%{init: {'theme': 'dark'}}%%
graph TB
    subgraph Container["🎨 Main Container"]
        Header["🧹 Cache Cleaner Header"]
        
        subgraph TabNav["Tab Navigation"]
            Tab1["🗂 Cache Cleaner"]
            Tab2["🔍 Smart Scanner"]
        end
        
        subgraph CacheView["Cache Cleaner View"]
            Total["💰 Total Size Display"]
            
            subgraph Sections["Cache Sections"]
                S1["📝 Editor Caches<br/>(VSCode, Cursor)"]
                S2["📦 Large Caches<br/>(>1GB folders)"]
                S3["💾 IndexedDB<br/>(Chrome origins)"]
                S4["🌐 Browser Caches<br/>(Chrome, Safari, Firefox, Arc)"]
                S5["📦 Package Managers<br/>(npm, yarn, pip, cargo...)"]
                S6["🔧 Dev Tools<br/>(Xcode caches)"]
                S7["📁 NPM Global Caches"]
                S8["⚙️ Custom Scanners"]
            end
        end
        
        subgraph SmartView["Smart Scanner View"]
            Filters["🎛️ Filter Controls<br/>(min size, min score)"]
            Suggestions["💡 Suggestions List<br/>(scored folders)"]
        end
        
        subgraph Actions["Action Buttons"]
            Scan["🔍 Scan"]
            Clean["🗑️ Clean Selected"]
        end
        
        Progress["📊 Progress Bar"]
        Status["📋 Status Messages"]
    end
    
    Header --> TabNav
    TabNav --> CacheView
    TabNav --> SmartView
    CacheView --> Total --> Sections
    SmartView --> Filters --> Suggestions
    Sections --> Actions
    Suggestions --> Actions
    Actions --> Progress --> Status
```

---

## 🔄 State Management Flow

```mermaid
%%{init: {'theme': 'dark'}}%%
stateDiagram-v2
    direction LR
    
    [*] --> Idle
    
    Idle --> Scanning: Click Scan
    Scanning --> Loaded: Scan Complete
    Scanning --> Error: Scan Failed
    
    Loaded --> Selecting: User Selection
    Selecting --> Loaded: Selection Changed
    
    Loaded --> Cleaning: Click Clean
    Selecting --> Cleaning: Click Clean
    
    Cleaning --> Progress: Start Clean
    Progress --> Progress: Update Progress
    Progress --> Completed: All Done
    Progress --> PartialError: Some Failed
    
    Completed --> Idle: Reset
    PartialError --> Idle: Acknowledge
    Error --> Idle: Retry
    
    note right of Scanning
        Async operation
        with progress
    end note
    
    note right of Cleaning
        Sequential clean
        with progress bar
    end note
```

---

## 📈 Performance Metrics

```mermaid
%%{init: {'theme': 'dark'}}%%
xychart-beta
    title "Cache Scan Performance (Estimated)"
    x-axis ["Browser", "Editor", "Package Mgr", "Xcode", "Large", "IndexedDB", "Smart"]
    y-axis "Time (ms)" 0 --> 2000
    bar [200, 150, 500, 800, 1500, 300, 1800]
    line [200, 150, 500, 800, 1500, 300, 1800]
```

---

## 🎪 Technology Stack Visualization

```mermaid
%%{init: {'theme': 'dark'}}%%
block-beta
    columns 3
    
    block:frontend["Frontend"]:1
        HTML["HTML5"]
        CSS["CSS3"]
        JS["JavaScript"]
    end
    
    block:bridge["Bridge"]:1
        Tauri["Tauri 2.0"]
        IPC["IPC Protocol"]
        WebView["WebView"]
    end
    
    block:backend["Backend"]:1
        Rust["Rust"]
        Tokio["Tokio"]
        Serde["Serde"]
    end
    
    frontend --> bridge --> backend
    
    style frontend fill:#3b82f6,stroke:#60a5fa
    style bridge fill:#8b5cf6,stroke:#a78bfa
    style backend fill:#f97316,stroke:#fb923c
```

---

## 📋 Summary

| Aspect | Details |
|--------|---------|
| **Framework** | Tauri 2.0 (Rust + WebView) |
| **Language** | Rust (Backend), HTML/JS (Frontend) |
| **Architecture** | Modular Scanner Pattern |
| **Async Runtime** | Tokio |
| **Supported Caches** | 22+ cache types |
| **Special Features** | Smart Suggestions with AI Scoring |
| **Security** | Full Disk Access, Dry Run, Whitelist |
| **Platform** | macOS (primary) |

