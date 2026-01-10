# Cache Cleaner App - Architecture Documentation

## 📋 Tổng Quan

Cache Cleaner App là một ứng dụng desktop được xây dựng bằng **Tauri** (Rust + HTML/JS), chuyên dụng cho việc quét và dọn dẹp các loại cache trên macOS. Ứng dụng sử dụng kiến trúc modular với các scanner chuyên biệt cho từng loại cache.

---

## 🏗️ High-Level Architecture

```mermaid
flowchart TB
    subgraph Frontend["Frontend (HTML/JS)"]
        UI[index.html]
        TabNav[Tab Navigation]
        CacheList[Cache List View]
        SmartScanner[Smart Scanner View]
    end

    subgraph TauriCore["Tauri Core"]
        InvokeHandler[Invoke Handler]
        Commands[Tauri Commands]
        Plugins[Plugins]
    end

    subgraph Backend["Backend (Rust)"]
        Main[main.rs]
        
        subgraph CacheModule["Cache Module"]
            Scanner[scanner.rs]
            Cleaner[cleaner.rs]
            Registry[ScannerRegistry]
        end
        
        subgraph Scanners["Specialized Scanners"]
            BrowserCaches[browser_caches.rs]
            DevTools[dev_tools.rs]
            PackageManagers[package_managers.rs]
            NpmCaches[npm_caches.rs]
            LargeCaches[large_caches.rs]
            IndexedDB[indexeddb.rs]
            CustomScanner[custom_scanner.rs]
            SmartSuggestions[smart_suggestions.rs]
        end
        
        subgraph Utils["Utils Module"]
            Filesystem[filesystem.rs]
            Permissions[permissions.rs]
            AccessTracker[access_tracker.rs]
            Concurrency[concurrency.rs]
        end
    end

    UI --> InvokeHandler
    InvokeHandler --> Commands
    Commands --> Main
    Main --> CacheModule
    CacheModule --> Scanners
    Scanners --> Utils
    Plugins --> Commands
```

---

## 📦 Module Structure

```mermaid
graph LR
    subgraph src-tauri/src
        main[main.rs]
        
        subgraph cache["cache/"]
            mod[mod.rs]
            scanner[scanner.rs]
            cleaner[cleaner.rs]
            registry[registry.rs]
            scanner_trait[scanner_trait.rs]
            parallel_scanner[parallel_scanner.rs]
            custom_scanner[custom_scanner.rs]
            smart_suggestions[smart_suggestions.rs]
            browser_caches[browser_caches.rs]
            dev_tools[dev_tools.rs]
            package_managers[package_managers.rs]
            npm_caches[npm_caches.rs]
            large_caches[large_caches.rs]
            indexeddb[indexeddb.rs]
            paths[paths.rs]
            config[config.rs]
            cache_dir[cache_dir.rs]
        end
        
        subgraph utils["utils/"]
            utils_mod[mod.rs]
            filesystem[filesystem.rs]
            permissions[permissions.rs]
            access_tracker[access_tracker.rs]
            concurrency[concurrency.rs]
        end
    end
    
    main --> mod
    main --> utils_mod
    mod --> scanner
    mod --> cleaner
    mod --> registry
```

---

## 🔄 Data Flow - Scan Operation

```mermaid
sequenceDiagram
    participant UI as Frontend (HTML/JS)
    participant Tauri as Tauri Handler
    participant Main as main.rs
    participant Scanner as scanner.rs
    participant Specialized as Specialized Scanners
    participant FS as filesystem.rs
    participant Disk as File System

    UI->>Tauri: invoke("scan_caches")
    Tauri->>Main: scan_caches()
    Main->>Scanner: scan_all()
    
    par Parallel Scanning
        Scanner->>Specialized: get_chrome_cache_info()
        Scanner->>Specialized: get_vscode_cache_info()
        Scanner->>Specialized: get_npm_cache_info()
        Scanner->>Specialized: get_xcode_cache_info()
    end
    
    Specialized->>FS: calculate_dir_size()
    FS->>Disk: read directory
    Disk-->>FS: directory contents
    FS-->>Specialized: size in bytes
    Specialized-->>Scanner: CacheInfo[]
    Scanner-->>Main: Vec<CacheInfo>
    Main-->>Tauri: JSON response
    Tauri-->>UI: Cache list data
```

---

## 🧹 Data Flow - Clean Operation

```mermaid
sequenceDiagram
    participant UI as Frontend (HTML/JS)
    participant Tauri as Tauri Handler
    participant Main as main.rs
    participant Cleaner as cleaner.rs
    participant FS as filesystem.rs
    participant Disk as File System

    UI->>Tauri: invoke("clean_cache", {cache_type})
    Tauri->>Main: clean_cache(cache_type, dry_run)
    Main->>Cleaner: clean(cache_type, dry_run)
    
    alt dry_run = true
        Cleaner-->>Main: CleanResult (simulated)
    else dry_run = false
        Cleaner->>FS: remove_dir_contents(path)
        FS->>Disk: delete files
        Disk-->>FS: result
        FS-->>Cleaner: bytes freed
        Cleaner-->>Main: CleanResult
    end
    
    Main-->>Tauri: JSON response
    Tauri-->>UI: Clean result
```

---

## 🎯 Cache Types Supported

```mermaid
mindmap
  root((Cache Types))
    Browsers
      Chrome
      Safari
      Firefox
      Arc
    Editors
      VSCode
      Cursor
    Package Managers
      NPM
      Yarn
      Pnpm
      Pip
      CocoaPods
      Cargo
      Gradle
    Xcode
      DerivedData
      Archives
      Simulators
    System
      SystemCaches
      UserLogs
      TempFiles
      IosBackups
    Special
      IndexedDB
      LargeCaches >1GB
      CustomScanner
```

---

## 🔧 Core Components

### 1. Scanner Trait System

```mermaid
classDiagram
    class CacheScanner {
        <<trait>>
        +id() String
        +display_name() String
        +scan() ScanResult
    }
    
    class CacheCleaner {
        <<trait>>
        +id() String
        +clean(dry_run) CleanResultGeneric
    }
    
    class ScanResult {
        +id: String
        +name: String
        +path: String
        +size_bytes: u64
        +item_count: usize
        +exists: bool
    }
    
    class CleanResultGeneric {
        +id: String
        +freed_bytes: u64
        +items_removed: usize
        +success: bool
        +message: String
        +dry_run: bool
    }
    
    class CustomScanner {
        +config: CustomScannerConfig
        +resolved_path: PathBuf
    }
    
    CacheScanner <|.. CustomScanner
    CacheCleaner <|.. CustomScanner
    CacheScanner --> ScanResult
    CacheCleaner --> CleanResultGeneric
```

### 2. Scanner Registry

```mermaid
classDiagram
    class ScannerRegistry {
        -custom_scanners: HashMap~String, CustomScanner~
        +new() ScannerRegistry
        +register(scanner: CustomScanner)
        +unregister(id: String)
        +get(id: String) Option~CustomScanner~
        +get_all() Vec~CustomScanner~
        +list() Vec~CustomScannerConfig~
        +scan_all_custom() Vec~ScanResult~
        +clean_custom(id, dry_run) CleanResultGeneric
    }
    
    class CustomScanner {
        +config: CustomScannerConfig
        +resolved_path: PathBuf
    }
    
    class CustomScannerConfig {
        +id: String
        +name: String
        +path: String
        +min_size_mb: Option~u64~
    }
    
    ScannerRegistry "1" *-- "*" CustomScanner
    CustomScanner --> CustomScannerConfig
```

---

## 🧠 Smart Suggestions System

```mermaid
flowchart TB
    subgraph Input["Input Sources"]
        Paths[Configurable Paths]
        Whitelist[Whitelist Paths]
    end
    
    subgraph Analysis["Analysis Engine"]
        FolderScanner[Folder Scanner]
        FeatureExtractor[Feature Extractor]
        ScoreCalculator[Score Calculator]
    end
    
    subgraph Features["Feature Extraction"]
        SizeMB[Size MB]
        LastAccess[Last Accessed Days]
        LocationType[Location Type]
    end
    
    subgraph Scoring["Score Components"]
        SizeScore[Size Score<br/>0-40 points]
        AgeScore[Age Score<br/>0-35 points]
        LocationScore[Location Score<br/>0-25 points]
    end
    
    subgraph Output["Output"]
        FolderSuggestion[FolderSuggestion]
        Reasons[Reasons Array]
    end
    
    Paths --> FolderScanner
    Whitelist -.->|exclude| FolderScanner
    FolderScanner --> FeatureExtractor
    FeatureExtractor --> Features
    Features --> ScoreCalculator
    ScoreCalculator --> Scoring
    SizeScore --> FolderSuggestion
    AgeScore --> FolderSuggestion
    LocationScore --> FolderSuggestion
    FolderSuggestion --> Reasons
```

### Score Calculation Logic

```mermaid
flowchart LR
    subgraph SizeScore["Size Score (0-40)"]
        S1[">10GB → 40"]
        S2["5-10GB → 35"]
        S3["1-5GB → 30"]
        S4["500MB-1GB → 20"]
        S5["100-500MB → 10"]
        S6["<100MB → 0"]
    end
    
    subgraph AgeScore["Age Score (0-35)"]
        A1[">365 days → 35"]
        A2["180-365 → 28"]
        A3["90-180 → 20"]
        A4["30-90 → 10"]
        A5["<30 → 0"]
    end
    
    subgraph LocationScore["Location Score (0-25)"]
        L1["Cache → 25"]
        L2["Log → 20"]
        L3["Build → 15"]
        L4["Dev → 10"]
        L5["Other → 0"]
    end
    
    SizeScore --> Total["Total Score<br/>(0-100)"]
    AgeScore --> Total
    LocationScore --> Total
```

---

## 🖥️ Tauri Commands API

```mermaid
flowchart TB
    subgraph BasicCommands["Basic Commands"]
        scan_caches["scan_caches()"]
        get_cache_size["get_cache_size(type)"]
        clean_cache["clean_cache(type, dry_run)"]
        check_permissions["check_permissions()"]
        check_chrome_running["check_chrome_running()"]
    end
    
    subgraph IndexedDBCommands["IndexedDB Commands"]
        scan_indexed_db["scan_indexed_db_items()"]
        clean_indexed_db["clean_indexed_db_items(paths)"]
    end
    
    subgraph LargeCacheCommands["Large Cache Commands"]
        scan_large["scan_large_caches()"]
        remove_large["remove_large_caches(paths)"]
    end
    
    subgraph NpmCommands["NPM Cache Commands"]
        scan_npm["scan_npm_caches()"]
        remove_npm["remove_npm_caches(paths)"]
    end
    
    subgraph CustomScannerCommands["Custom Scanner Commands"]
        register_custom["register_custom_scanner(config)"]
        list_custom["list_custom_scanners()"]
        remove_custom["remove_custom_scanner(id)"]
        scan_custom["scan_custom_caches()"]
        clean_custom["clean_custom_cache(id, dry_run)"]
    end
    
    subgraph SmartSuggestionsCommands["Smart Suggestions Commands"]
        scan_smart["scan_smart_suggestions()"]
        get_folder_info["get_folder_suggestion_info(path)"]
        remove_smart["remove_smart_suggestions(paths)"]
    end
```

---

## 🔐 Utils Module

```mermaid
flowchart LR
    subgraph Filesystem["filesystem.rs"]
        calc_dir[calculate_dir_size]
        calc_file[calculate_file_size]
        count_items[count_items]
        remove_dir[remove_dir_contents]
        remove_file[remove_file]
    end
    
    subgraph Permissions["permissions.rs"]
        chrome_running[is_chrome_running]
        disk_access[has_full_disk_access]
        home_access[can_access_home]
    end
    
    subgraph AccessTracker["access_tracker.rs"]
        folder_info[FolderAccessInfo]
        get_access[get_access_info]
        days_since[days_since_access]
    end
    
    subgraph Concurrency["concurrency.rs"]
        semaphore[create_semaphore]
        default_conc[DEFAULT_CONCURRENCY]
    end
```

---

## 🎨 Frontend Architecture

```mermaid
flowchart TB
    subgraph UI["index.html"]
        subgraph TabSystem["Tab Navigation"]
            CacheTab[Cache Cleaner Tab]
            SmartTab[Smart Scanner Tab]
        end
        
        subgraph CacheCleanerView["Cache Cleaner View"]
            TotalSize[Total Size Display]
            BasicSection[Editor Caches Section]
            LargeSection[Large Caches Section]
            IndexedSection[IndexedDB Section]
            BrowserSection[Browser Caches Section]
            PackageSection[Package Managers Section]
            DevToolsSection[Dev Tools Section]
            NpmSection[NPM Caches Section]
            CustomSection[Custom Scanners Section]
        end
        
        subgraph SmartScannerView["Smart Scanner View"]
            FilterControls[Filter Controls]
            SuggestionsList[Suggestions List]
            ScoreBadge[Score Badges]
            ReasonsTags[Reasons Tags]
        end
        
        subgraph Actions["Actions"]
            ScanBtn[Scan Button]
            CleanBtn[Clean Button]
            ProgressBar[Progress Bar]
        end
    end
    
    TabSystem --> CacheCleanerView
    TabSystem --> SmartScannerView
    CacheCleanerView --> Actions
    SmartScannerView --> Actions
```

---

## 📊 Data Models

```mermaid
erDiagram
    CacheInfo {
        CacheType cache_type
        String path
        u64 size
        bool exists
        usize item_count
    }
    
    CleanResult {
        CacheType cache_type
        u64 freed_bytes
        usize items_removed
        bool success
        String message
        bool dry_run
    }
    
    IndexedDbItem {
        String profile
        String origin
        String path
        u64 size
        bool over_threshold
    }
    
    LargeCacheEntry {
        String name
        String path
        u64 size_bytes
    }
    
    NpmCacheEntry {
        String name
        String path
        u64 size_bytes
        String relative_path
    }
    
    FolderSuggestion {
        String path
        String name
        u64 size_bytes
        f64 score
        Vec reasons
        u64 last_accessed_days_ago
    }
    
    CustomScannerConfig {
        String id
        String name
        String path
        Option min_size_mb
    }
```

---

## 🚀 Build & Deployment

```mermaid
flowchart LR
    subgraph Development
        RustCode[Rust Code]
        HTMLCode[HTML/CSS/JS]
        TauriConfig[tauri.conf.json]
    end
    
    subgraph Build["Build Process"]
        CargoBuild[cargo build]
        TauriBuild[tauri build]
    end
    
    subgraph Output["Output"]
        DebugBinary[Debug Binary]
        ReleaseBinary[Release Binary]
        AppBundle[.app Bundle]
        DMG[DMG Installer]
    end
    
    RustCode --> CargoBuild
    HTMLCode --> TauriBuild
    TauriConfig --> TauriBuild
    CargoBuild --> DebugBinary
    TauriBuild --> ReleaseBinary
    TauriBuild --> AppBundle
    TauriBuild --> DMG
```

---

## 🔒 Security Considerations

```mermaid
flowchart TB
    subgraph Permissions["Permission Checks"]
        FDA[Full Disk Access Check]
        HomeAccess[Home Directory Access]
        ProcessCheck[Chrome Running Check]
    end
    
    subgraph Protection["Protection Mechanisms"]
        DryRun[Dry Run Mode]
        Whitelist[Whitelist Paths]
        SizeThreshold[Size Thresholds]
    end
    
    subgraph Capabilities["Tauri Capabilities"]
        Shell[Shell Plugin]
        Dialog[Dialog Plugin]
        FS[File System Access]
    end
    
    Permissions --> Protection
    Protection --> Capabilities
```

---

## 📈 Performance Optimizations

- **Parallel Scanning**: Sử dụng `tokio` async runtime với semaphore để giới hạn concurrent operations
- **Lazy Loading**: Chỉ load cache info khi cần thiết
- **Incremental Updates**: Progress bar updates trong quá trình clean
- **Efficient Directory Walking**: Sử dụng `std::fs` với recursion optimization

---

## 🔗 Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` | Desktop app framework |
| `serde` | Serialization/Deserialization |
| `tokio` | Async runtime |
| `dirs` | Cross-platform directory paths |
| `lazy_static` | Static initialization |

---

## 📝 Notes

- Ứng dụng yêu cầu **Full Disk Access** trên macOS để truy cập một số thư mục cache
- Chrome phải được đóng trước khi xóa Chrome cache
- Smart Suggestions sử dụng scoring algorithm để đề xuất folders nên xóa
- Custom Scanner cho phép user định nghĩa paths tùy chỉnh để scan

