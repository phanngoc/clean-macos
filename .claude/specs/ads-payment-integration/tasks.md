# Task Breakdown: Ads & Payment Integration

## Task Overview
Total estimated tasks: 25
Estimated total effort: 120-160 hours (3-4 weeks for 1 developer)

## Task Dependencies Graph
```
Research Tasks (1-3) → Foundation Tasks (4-6) → Backend Tasks (7-12) 
                                                   ↓
Frontend Tasks (13-18) ← Integration Tasks (19-22) ← Payment Tasks (7-9)
                                                   ↓
Testing Tasks (23-24) → Documentation Task (25)
```

---

## 📦 Phase 1: Research Tasks

### Task #1: Research Ad Integration Libraries for macOS/Tauri

**Category**: Research

**Description**:
Research and evaluate available ad SDKs that can be integrated into a Tauri desktop application. Focus on:
- Compatibility with Tauri framework and web-based frontend
- Support for macOS desktop applications (not just mobile)
- 15-second video ad formats
- Revenue models (CPM, CPC, revenue share percentages)
- Integration complexity and documentation quality
- Privacy and compliance (GDPR, CCPA)
- Ad quality and user experience
- Minimum payout thresholds and payment schedules

**Expected Output**:
- Research document comparing 3-5 ad providers (Google AdMob, Unity Ads, AppLovin, AdColony, etc.)
- Recommendation matrix with pros/cons for each
- Selected ad provider with justification
- Integration requirements and prerequisites list

**Input Requirements**:
- Access to ad provider documentation
- Understanding of Tauri architecture
- Knowledge of web ad integration patterns

**Dependencies**:
- None

**Acceptance Criteria**:
- [x] At least 3 ad providers researched and documented
- [x] Compatibility with Tauri confirmed for selected provider
- [x] 15-second video ad format supported
- [x] Revenue model and rates documented
- [x] Integration complexity assessed
- [x] Privacy compliance verified

**Complexity**: Medium
**Estimated Time**: 8-12 hours

**Implementation Notes**:
- Check Tauri community forums for existing ad integrations
- Test ad SDK in simple Tauri test app if possible
- Contact ad provider support for desktop app support confirmation

**Status**: ✅ **COMPLETED**
- Research document created: `research-ad-providers.md`
- 5 ad providers evaluated (Google AdSense, Unity Ads, AppLovin, AdColony, Custom Web)
- Recommendation: Google AdSense via web integration (best Tauri compatibility)
- All acceptance criteria met

---

### Task #2: Research Payment Integration Libraries for macOS/Tauri

**Category**: Research

**Description**:
Research and evaluate payment providers suitable for one-time $15 purchases in a macOS desktop application. Consider:
- Support for desktop applications (not just web/mobile)
- One-time payment support (not just subscriptions)
- Integration with Tauri/Rust backend
- Transaction fees and pricing models
- Security and PCI compliance
- Receipt validation mechanisms
- Support for Apple Pay and other native payment methods
- macOS App Store In-App Purchase (if applicable)
- Documentation quality and developer experience

**Expected Output**:
- Research document comparing 3-5 payment providers (Stripe, Paddle, RevenueCat, Apple IAP, etc.)
- Recommendation matrix with pros/cons
- Selected payment provider(s) with justification
- Integration requirements and API documentation
- Fee structure analysis

**Input Requirements**:
- Access to payment provider documentation
- Understanding of payment processing flow
- Knowledge of Tauri IPC and Rust async patterns

**Dependencies**:
- None

**Acceptance Criteria**:
- [x] At least 3 payment providers researched (5 providers: Paddle, Stripe, Apple IAP, RevenueCat, PayPal)
- [x] One-time payment support confirmed (all providers support one-time payments)
- [x] Tauri/Rust integration feasibility verified (integration paths documented for all providers)
- [x] Transaction fees documented and compared (detailed fee analysis with revenue comparison)
- [x] Security and compliance verified (all providers are PCI compliant)
- [x] Receipt validation approach documented (each provider's validation method documented)
- [x] Integration complexity assessed (complexity ranking and time estimates provided)

**Complexity**: Medium
**Estimated Time**: 8-12 hours

**Implementation Notes**:
- Consider both App Store and direct distribution scenarios
- Evaluate if multiple payment providers needed (App Store vs direct)
- Check for existing Tauri payment integration examples
- Verify PCI compliance requirements

---

### Task #3: Create Integration Architecture Design

**Category**: Research/Design

**Description**:
Based on research from Tasks #1 and #2, create detailed technical architecture for integrating selected ad and payment providers into the existing Cache Cleaner application. Design:
- Component structure and interfaces
- Data flow diagrams
- Error handling strategies
- Security considerations
- Performance optimization approaches

**Expected Output**:
- Detailed architecture document (can reference design.md)
- Component interface definitions (Rust structs/traits)
- Sequence diagrams for key flows
- Error handling flowcharts
- Security architecture diagram

**Input Requirements**:
- Results from Task #1 (ad provider research)
- Results from Task #2 (payment provider research)
- Existing codebase structure understanding
- Design.md document (if already created)

**Dependencies**:
- Task #1 (Research Ad Integration Libraries)
- Task #2 (Research Payment Integration Libraries)

**Acceptance Criteria**:
- [x] Architecture document created with all components defined
- [x] Interfaces designed for ad and payment systems
- [x] Integration points with existing code identified
- [x] Error handling strategies documented
- [x] Security measures defined
- [x] Performance considerations addressed

**Complexity**: Medium
**Estimated Time**: 6-8 hours

**Implementation Notes**:
- Review existing main.rs and cache module structure
- Ensure architecture aligns with Tauri best practices
- Consider future extensibility (multiple providers)

**Status**: ✅ **COMPLETED**
- Architecture document updated: `design.md`
- All components defined with detailed Rust struct/trait definitions:
  - AdManager (Google AdSense integration)
  - PaymentManager (Paddle integration)
  - PremiumService (encrypted storage)
  - MonetizationMiddleware (integration with existing cache cleaner)
- Integration points documented:
  - Modified `main.rs` with new Tauri commands
  - Integration with existing `cache::cleaner::clean()` function
  - Frontend component structure
- Error handling flowcharts created (text-based)
- Security architecture diagram created
- Performance optimization strategies documented
- Based on research results from Task #1 (Google AdSense) and Task #2 (Paddle)

---

## 📦 Phase 2: Foundation Tasks

### Task #4: Set Up Ad Provider SDK and Account

**Category**: Setup

**Description**:
Set up developer account with selected ad provider and integrate SDK into the project. This includes:
- Creating developer account
- Registering application
- Obtaining API keys and ad unit IDs
- Adding SDK dependencies to project (frontend)
- Configuring ad provider dashboard
- Setting up test ad units for development

**Expected Output**:
- Ad provider account created and configured
- SDK integrated into frontend (package.json or equivalent)
- API keys stored securely (environment variables)
- Test ad units configured
- Basic ad loading test working

**Input Requirements**:
- Selected ad provider from Task #1
- Project access and build system understanding

**Dependencies**:
- Task #1 (Research Ad Integration Libraries)

**Acceptance Criteria**:
- [x] Ad provider account created
- [x] Application registered with provider
- [x] SDK added to project dependencies
- [x] API keys configured (not hardcoded)
- [x] Test ad unit created
- [x] Can load test ad in development environment

**Complexity**: Low
**Estimated Time**: 2-4 hours

**Implementation Notes**:
- Store API keys in .env file (not committed to git)
- Use test ad units during development
- Document API key setup in README

**Status**: ✅ **COMPLETED**
- Google AdSense SDK integrated into frontend (`ui/index.html`)
- AdSense script tag added to HTML head
- Ad loading functionality implemented with test ad support
- Configuration object created for easy API key management
- Test ad loading function available (`testAdLoading()`)
- Setup documentation created (`ADSENSE_SETUP.md`)
- README updated with AdSense setup instructions
- `.gitignore` already configured to exclude `.env` files
- Test ad unit ID configured for development (Google's test ad unit)

---

### Task #5: Set Up Payment Provider SDK and Account

**Category**: Setup

**Description**:
Set up developer account with selected payment provider and integrate SDK into the project. This includes:
- Creating developer account
- Registering application/product ($15 premium license)
- Obtaining API keys and product IDs
- Adding SDK dependencies (Rust crates for backend)
- Configuring webhook endpoints (if needed)
- Setting up test mode for development

**Expected Output**:
- Payment provider account created and configured
- Product created ($15 premium license)
- SDK integrated into Rust backend (Cargo.toml)
- API keys stored securely
- Test mode configured
- Basic payment test working

**Input Requirements**:
- Selected payment provider from Task #2
- Project access and Rust/Cargo understanding

**Dependencies**:
- Task #2 (Research Payment Integration Libraries)

**Acceptance Criteria**:
- [ ] Payment provider account created
- [ ] Product created with $15 price
- [ ] SDK added to Cargo.toml
- [ ] API keys configured securely
- [ ] Test mode enabled
- [ ] Can initiate test payment in development

**Complexity**: Low
**Estimated Time**: 2-4 hours

**Implementation Notes**:
- Use test API keys during development
- Create separate test and production products
- Document payment provider setup

---

### Task #6: Create Local Storage Module for Premium Status

**Category**: Data/Schema

**Description**:
Create a Rust module for storing and retrieving premium status from local encrypted storage. This includes:
- Define PremiumStatus struct with all required fields
- Implement encryption/decryption for sensitive data
- Create storage interface (file-based or keychain)
- Implement read/write operations
- Add error handling for corrupted data
- Create migration path for future schema changes

**Expected Output**:
- `src/monetization/storage.rs` module
- PremiumStatus struct definition
- Encryption utilities
- Storage read/write functions
- Error types for storage operations
- Unit tests for storage operations

**Input Requirements**:
- Rust project structure
- Encryption library (e.g., `aes-gcm` or `ring`)

**Dependencies**:
- None (can be done in parallel with other tasks)

**Acceptance Criteria**:
- [ ] PremiumStatus struct defined with all fields
- [ ] Encryption implemented for sensitive data
- [ ] Storage read/write functions working
- [ ] Error handling for corrupted/missing data
- [ ] Unit tests passing (>80% coverage)
- [ ] Storage location documented

**Complexity**: Medium
**Estimated Time**: 6-8 hours

**Implementation Notes**:
- Use JSON for storage format (easy to debug)
- Encrypt receipt data and transaction IDs
- Store in app data directory (platform-specific)
- Handle file permissions correctly

---

## 📦 Phase 3: Backend/API Tasks

### Task #7: Implement PaymentManager Service

**Category**: Backend/API

**Description**:
Implement the PaymentManager service in Rust that handles payment processing. This includes:
- Create PaymentManager struct with payment provider client
- Implement `initiate_purchase()` method
- Implement `process_payment()` method
- Implement `validate_receipt()` method
- Implement `restore_purchases()` method
- Add error handling and retry logic
- Add logging for payment events

**Expected Output**:
- `src/monetization/payment_manager.rs` module
- PaymentManager struct and implementation
- Integration with selected payment provider SDK
- Error types for payment operations
- Unit tests for payment flows

**Input Requirements**:
- Payment provider SDK from Task #5
- Payment provider API credentials
- Understanding of async Rust and error handling

**Dependencies**:
- Task #5 (Set Up Payment Provider SDK)
- Task #6 (Create Local Storage Module)

**Acceptance Criteria**:
- [ ] PaymentManager struct implemented
- [ ] Can initiate payment session
- [ ] Can process payment transaction
- [ ] Can validate purchase receipts
- [ ] Can restore previous purchases
- [ ] Error handling covers all failure cases
- [ ] Unit tests passing
- [ ] Integration tests with test payment provider passing

**Complexity**: High
**Estimated Time**: 12-16 hours

**Implementation Notes**:
- Use async/await for payment API calls
- Implement proper error types (use `thiserror`)
- Add retry logic with exponential backoff
- Log all payment events (without sensitive data)
- Handle network timeouts gracefully

---

### Task #8: Implement PremiumService

**Category**: Backend/API

**Description**:
Implement the PremiumService that manages premium user status. This includes:
- Create PremiumService struct
- Implement `is_premium()` method (checks local storage)
- Implement `grant_premium()` method (stores status after purchase)
- Implement `revoke_premium()` method (removes premium status)
- Implement `verify_premium_status()` method (remote verification)
- Add periodic verification logic
- Integrate with PaymentManager for receipt validation

**Expected Output**:
- `src/monetization/premium_service.rs` module
- PremiumService struct and implementation
- Integration with LocalStorage and PaymentManager
- Verification logic
- Unit tests

**Input Requirements**:
- Local storage module from Task #6
- PaymentManager from Task #7
- Understanding of premium status lifecycle

**Dependencies**:
- Task #6 (Create Local Storage Module)
- Task #7 (Implement PaymentManager Service)

**Acceptance Criteria**:
- [ ] PremiumService implemented
- [ ] Can check premium status (local)
- [ ] Can grant premium status after purchase
- [ ] Can revoke premium status
- [ ] Can verify premium status remotely
- [ ] Handles corrupted/missing status gracefully
- [ ] Unit tests passing
- [ ] Integration with PaymentManager working

**Complexity**: Medium
**Estimated Time**: 8-10 hours

**Implementation Notes**:
- Cache premium status in memory for performance
- Verify status periodically (e.g., on app launch, weekly)
- Handle network failures during verification
- Use atomic operations for status updates

---

### Task #9: Implement AdManager Service

**Category**: Backend/API

**Description**:
Implement the AdManager service that manages ad lifecycle. This includes:
- Create AdManager struct
- Implement `request_ad()` method (coordinates with frontend)
- Implement `wait_for_ad_completion()` method
- Implement `is_ad_required()` method (checks premium status)
- Add ad event tracking and logging
- Add error handling for ad failures

**Expected Output**:
- `src/monetization/ad_manager.rs` module
- AdManager struct and implementation
- Integration with PremiumService
- Ad event tracking
- Unit tests

**Input Requirements**:
- PremiumService from Task #8
- Understanding of ad lifecycle
- Frontend ad SDK integration (from Task #4)

**Dependencies**:
- Task #8 (Implement PremiumService)
- Task #4 (Set Up Ad Provider SDK) - for understanding ad flow

**Acceptance Criteria**:
- [ ] AdManager implemented
- [ ] Can request ad (coordinates with frontend)
- [ ] Can track ad completion
- [ ] Can check if ad is required (based on premium status)
- [ ] Ad events logged for analytics
- [ ] Error handling for ad failures
- [ ] Unit tests passing

**Complexity**: Medium
**Estimated Time**: 6-8 hours

**Implementation Notes**:
- AdManager mainly coordinates - actual ad display in frontend
- Use channels or callbacks for ad completion events
- Track ad metrics (load time, completion rate, errors)

---

### Task #10: Create MonetizationMiddleware

**Category**: Backend/API

**Description**:
Create middleware that intercepts deletion requests and enforces ad/payment requirements. This includes:
- Modify existing `clean_cache` command or create wrapper
- Check premium status before deletion
- If not premium: signal frontend to show ad, wait for completion
- If premium: proceed immediately
- Integrate with existing cache cleaner
- Add logging for monetization events

**Expected Output**:
- Modified `main.rs` with monetization middleware
- New Tauri command: `clean_cache_with_monetization()`
- Integration with AdManager and PremiumService
- Updated command handler registration

**Input Requirements**:
- Existing `clean_cache` command
- AdManager from Task #9
- PremiumService from Task #8

**Dependencies**:
- Task #8 (Implement PremiumService)
- Task #9 (Implement AdManager Service)

**Acceptance Criteria**:
- [ ] Monetization middleware implemented
- [ ] Checks premium status before deletion
- [ ] Blocks deletion until ad completes (free users)
- [ ] Allows immediate deletion (premium users)
- [ ] Integrates with existing cache cleaner
- [ ] Logging added for monetization events
- [ ] Unit tests passing

**Complexity**: Medium
**Estimated Time**: 6-8 hours

**Implementation Notes**:
- Can modify existing `clean_cache` or create wrapper
- Use async/await for ad completion waiting
- Maintain backward compatibility if possible
- Add feature flag to enable/disable monetization

---

### Task #11: Add Tauri Commands for Monetization

**Category**: Backend/API

**Description**:
Add all Tauri command handlers for monetization features. This includes:
- `check_premium_status()` - Check if user is premium
- `request_ad()` - Request ad for display
- `ad_completed()` - Notify backend of ad completion
- `initiate_purchase()` - Start payment flow
- `process_payment()` - Process payment transaction
- `restore_purchases()` - Restore previous purchases
- Register all commands in main.rs

**Expected Output**:
- Tauri command handlers in `main.rs`
- All commands registered in `invoke_handler!` macro
- Command signatures match API contracts from design.md
- Error handling for all commands

**Input Requirements**:
- PaymentManager from Task #7
- PremiumService from Task #8
- AdManager from Task #9
- Understanding of Tauri command system

**Dependencies**:
- Task #7 (Implement PaymentManager Service)
- Task #8 (Implement PremiumService)
- Task #9 (Implement AdManager Service)

**Acceptance Criteria**:
- [ ] All Tauri commands implemented
- [ ] Commands registered in main.rs
- [ ] Error handling implemented
- [ ] Command signatures match design.md
- [ ] Can invoke commands from frontend
- [ ] Integration tests passing

**Complexity**: Low-Medium
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Follow existing Tauri command patterns in main.rs
- Use proper error types and serialization
- Add command documentation comments

---

### Task #12: Add Error Types and Error Handling

**Category**: Backend/API

**Description**:
Create comprehensive error types for monetization features and implement error handling throughout. This includes:
- Define error enums for AdError, PaymentError, PremiumError
- Implement error conversion traits
- Add user-friendly error messages
- Implement retry logic where appropriate
- Add error logging

**Expected Output**:
- `src/monetization/errors.rs` module
- Error types for all monetization operations
- Error handling in all services
- Error conversion implementations

**Input Requirements**:
- Understanding of Rust error handling (`thiserror`, `anyhow`)
- All monetization services implemented

**Dependencies**:
- Task #7 (Implement PaymentManager Service)
- Task #8 (Implement PremiumService)
- Task #9 (Implement AdManager Service)

**Acceptance Criteria**:
- [ ] Error types defined for all operations
- [ ] Error messages are user-friendly
- [ ] Retry logic implemented where needed
- [ ] Errors logged appropriately
- [ ] Error conversion traits implemented

**Complexity**: Low
**Estimated Time**: 3-4 hours

**Implementation Notes**:
- Use `thiserror` for error definitions
- Provide context in error messages
- Don't expose sensitive information in errors

---

## 📦 Phase 4: Frontend/UI Tasks

### Task #13: Create AdDisplayComponent

**Category**: Frontend/UI

**Description**:
Create React/Vue component for displaying advertisements. This includes:
- Create AdDisplay component
- Integrate ad provider SDK
- Display ad container with proper styling
- Show 15-second countdown timer
- Prevent ad dismissal before completion
- Handle ad loading states (loading, error, playing)
- Emit completion event to backend
- Handle ad errors and retries

**Expected Output**:
- `src/components/AdDisplay.tsx` (or .vue)
- Ad display UI with timer
- Integration with ad provider SDK
- Error handling UI
- Styling for ad container

**Input Requirements**:
- Ad provider SDK from Task #4
- Frontend framework (React/Vue)
- Tauri IPC client

**Dependencies**:
- Task #4 (Set Up Ad Provider SDK)
- Task #11 (Add Tauri Commands) - for ad_completed command

**Acceptance Criteria**:
- [ ] AdDisplay component created
- [ ] Can load and display ads
- [ ] 15-second timer displayed
- [ ] Ad cannot be dismissed before completion
- [ ] Completion event sent to backend
- [ ] Error states handled with retry option
- [ ] Responsive styling
- [ ] Accessibility considerations (if applicable)

**Complexity**: Medium
**Estimated Time**: 8-10 hours

**Implementation Notes**:
- Use modal/overlay for ad display
- Make ad container fullscreen or prominent
- Add loading spinner while ad loads
- Show error message if ad fails to load
- Test with test ad units

---

### Task #14: Create PaymentComponent

**Category**: Frontend/UI

**Description**:
Create React/Vue component for premium purchase flow. This includes:
- Create PaymentComponent
- Display premium purchase options ($15)
- Show payment form or integrate payment provider UI
- Handle payment submission
- Display payment status (processing, success, error)
- Show purchase confirmation
- Handle payment errors with retry option

**Expected Output**:
- `src/components/PaymentComponent.tsx` (or .vue)
- Payment form/UI
- Integration with payment provider SDK (if web-based)
- Payment status indicators
- Success/error states

**Input Requirements**:
- Payment provider SDK (if web-based UI)
- Frontend framework
- Tauri IPC client

**Dependencies**:
- Task #5 (Set Up Payment Provider SDK)
- Task #11 (Add Tauri Commands) - for payment commands

**Acceptance Criteria**:
- [ ] PaymentComponent created
- [ ] Can display payment options
- [ ] Can submit payment
- [ ] Payment status displayed (processing, success, error)
- [ ] Purchase confirmation shown
- [ ] Error handling with retry
- [ ] Responsive styling
- [ ] Security considerations (no sensitive data in logs)

**Complexity**: Medium-High
**Estimated Time**: 10-12 hours

**Implementation Notes**:
- Use payment provider's secure payment UI if available
- Never store payment data in frontend
- Show clear pricing ($15 USD)
- Add loading states during payment processing
- Test with test payment mode

---

### Task #15: Add Premium Status Indicator

**Category**: Frontend/UI

**Description**:
Add UI indicators throughout the app to show premium status. This includes:
- Premium badge/indicator in header/navbar
- Update delete buttons to show "Watch Ad to Delete" for free users
- Show "Delete" (no ad) for premium users
- Add "Upgrade to Premium" button/option
- Update UI based on premium status changes

**Expected Output**:
- Premium status indicator component
- Updated delete buttons with ad indicators
- Upgrade to Premium button/option
- UI updates based on premium status

**Input Requirements**:
- Existing UI components
- Tauri IPC client for checking premium status

**Dependencies**:
- Task #11 (Add Tauri Commands) - for check_premium_status

**Acceptance Criteria**:
- [ ] Premium badge displayed for premium users
- [ ] Delete buttons show appropriate text based on status
- [ ] Upgrade option visible for free users
- [ ] UI updates when premium status changes
- [ ] Consistent styling with app design

**Complexity**: Low
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Check premium status on component mount
- Listen for premium status changes (if real-time updates needed)
- Use consistent styling/colors for premium indicators

---

### Task #16: Integrate Ad Display into Deletion Flow

**Category**: Frontend/UI

**Description**:
Integrate AdDisplay component into the deletion workflow. This includes:
- Modify delete button handlers to check premium status
- If not premium: show AdDisplay modal before deletion
- Wait for ad completion before proceeding with deletion
- If premium: proceed with deletion immediately
- Update UI feedback during ad playback
- Handle ad errors in deletion flow

**Expected Output**:
- Updated delete button handlers
- AdDisplay integrated into deletion flow
- Seamless user experience (ad → deletion)

**Input Requirements**:
- AdDisplay component from Task #13
- Existing deletion button handlers
- Premium status checking

**Dependencies**:
- Task #13 (Create AdDisplayComponent)
- Task #15 (Add Premium Status Indicator)

**Acceptance Criteria**:
- [ ] Delete buttons trigger ad for free users
- [ ] Ad displays before deletion
- [ ] Deletion proceeds after ad completion
- [ ] Premium users skip ad
- [ ] Error handling if ad fails
- [ ] User experience is smooth

**Complexity**: Medium
**Estimated Time**: 6-8 hours

**Implementation Notes**:
- Use modal/overlay for ad display
- Disable delete button during ad playback
- Show progress indicator during deletion after ad

---

### Task #17: Integrate Payment Flow into UI

**Category**: Frontend/UI

**Description**:
Integrate PaymentComponent into the app UI. This includes:
- Add "Upgrade to Premium" button/option in settings or header
- Show PaymentComponent modal when clicked
- Handle payment success (update UI, show confirmation)
- Handle payment errors
- Update premium status indicator after purchase

**Expected Output**:
- PaymentComponent integrated into app
- Upgrade button/option added
- Payment flow working end-to-end

**Input Requirements**:
- PaymentComponent from Task #14
- App UI structure

**Dependencies**:
- Task #14 (Create PaymentComponent)
- Task #15 (Add Premium Status Indicator)

**Acceptance Criteria**:
- [ ] Upgrade option accessible from UI
- [ ] PaymentComponent displays when triggered
- [ ] Payment success updates UI immediately
- [ ] Premium status indicator updates after purchase
- [ ] Error handling works

**Complexity**: Low-Medium
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Add upgrade option in settings menu or prominent location
- Show payment modal/overlay
- Refresh premium status after successful purchase

---

### Task #18: Add Loading States and Error Handling UI

**Category**: Frontend/UI

**Description**:
Add comprehensive loading states and error handling UI for all monetization features. This includes:
- Loading spinners for ad loading, payment processing
- Error messages for ad failures, payment failures
- Retry buttons for failed operations
- Success messages for completed operations
- Toast notifications or inline messages

**Expected Output**:
- Loading state components
- Error message components
- Toast/notification system (if not exists)
- Error handling throughout monetization UI

**Input Requirements**:
- All monetization UI components
- Existing UI patterns (if any)

**Dependencies**:
- Task #13 (Create AdDisplayComponent)
- Task #14 (Create PaymentComponent)
- Task #16 (Integrate Ad Display into Deletion Flow)
- Task #17 (Integrate Payment Flow into UI)

**Acceptance Criteria**:
- [ ] Loading states for all async operations
- [ ] Error messages for all error cases
- [ ] Retry options where appropriate
- [ ] Success feedback for completed operations
- [ ] Consistent error handling patterns

**Complexity**: Low
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Use existing UI patterns if available
- Make error messages user-friendly
- Provide actionable error messages (e.g., "Retry" button)

---

## 📦 Phase 5: Integration Tasks

### Task #19: Integrate Monetization with All Deletion Operations

**Category**: Integration

**Description**:
Ensure monetization (ad requirement) is integrated with all deletion operations in the app. This includes:
- Review all deletion entry points (clean_cache, remove_large_caches, remove_npm_caches, etc.)
- Ensure all deletion operations check premium status
- Ensure all deletion operations show ad for free users
- Test all deletion flows with and without premium

**Expected Output**:
- All deletion operations integrated with monetization
- Consistent behavior across all deletion types
- Updated deletion handlers

**Input Requirements**:
- All existing deletion commands
- MonetizationMiddleware from Task #10

**Dependencies**:
- Task #10 (Create MonetizationMiddleware)
- Task #16 (Integrate Ad Display into Deletion Flow)

**Acceptance Criteria**:
- [ ] All deletion operations check premium status
- [ ] All deletion operations show ad for free users
- [ ] Premium users skip ads for all deletion types
- [ ] Consistent behavior across app
- [ ] All deletion flows tested

**Complexity**: Medium
**Estimated Time**: 6-8 hours

**Implementation Notes**:
- List all deletion entry points first
- Create wrapper or modify each deletion handler
- Test each deletion type individually
- Ensure dry-run mode still works (may skip ads)

---

### Task #20: Implement Premium Status Verification on App Launch

**Category**: Integration

**Description**:
Implement premium status verification when app launches. This includes:
- Check premium status from local storage on app start
- Verify premium status with payment provider (remote verification)
- Update premium status if verification fails
- Show restore purchase option if status lost
- Handle offline scenarios (use cached status)

**Expected Output**:
- App launch verification logic
- Premium status sync with payment provider
- Restore purchase functionality
- Offline handling

**Input Requirements**:
- PremiumService from Task #8
- PaymentManager from Task #7
- App initialization code

**Dependencies**:
- Task #7 (Implement PaymentManager Service)
- Task #8 (Implement PremiumService)

**Acceptance Criteria**:
- [ ] Premium status checked on app launch
- [ ] Remote verification performed (if online)
- [ ] Status updated if verification fails
- [ ] Restore purchase option available
- [ ] Works offline (uses cached status)

**Complexity**: Medium
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Run verification in background (don't block app startup)
- Show loading indicator if verification takes time
- Cache verification result to avoid repeated checks

---

### Task #21: Add Analytics and Logging

**Category**: Integration

**Description**:
Add comprehensive analytics and logging for monetization features. This includes:
- Log ad events (loaded, started, completed, failed)
- Log payment events (initiated, completed, failed)
- Log premium status changes
- Track conversion metrics (ad views, premium purchases)
- Add analytics events for business intelligence

**Expected Output**:
- Analytics/logging system integrated
- Monetization events logged
- Metrics tracked
- Log files or analytics dashboard (if applicable)

**Input Requirements**:
- Analytics service (if using external service)
- Logging infrastructure

**Dependencies**:
- All monetization services implemented

**Acceptance Criteria**:
- [ ] Ad events logged
- [ ] Payment events logged
- [ ] Premium status changes logged
- [ ] Conversion metrics tracked
- [ ] No sensitive data in logs
- [ ] Logs useful for debugging and analytics

**Complexity**: Low-Medium
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Use structured logging
- Don't log sensitive payment data
- Consider privacy regulations (GDPR) for analytics
- Use analytics service if available (e.g., Mixpanel, Amplitude)

---

### Task #22: Add Configuration and Feature Flags

**Category**: Integration

**Description**:
Add configuration system for monetization features. This includes:
- Feature flag to enable/disable monetization
- Configuration for ad provider settings
- Configuration for payment provider settings
- Environment-based configuration (dev, staging, production)
- Configuration file or environment variables

**Expected Output**:
- Configuration system for monetization
- Feature flags implemented
- Environment-based configs
- Configuration documentation

**Input Requirements**:
- Understanding of app configuration system

**Dependencies**:
- All monetization features implemented

**Acceptance Criteria**:
- [ ] Feature flag to enable/disable monetization
- [ ] Ad provider configurable
- [ ] Payment provider configurable
- [ ] Different configs for dev/prod
- [ ] Configuration documented

**Complexity**: Low
**Estimated Time**: 3-4 hours

**Implementation Notes**:
- Use environment variables for sensitive config (API keys)
- Use config file for non-sensitive settings
- Make it easy to switch between test and production modes

---

## 📦 Phase 6: Testing Tasks

### Task #23: Write Unit Tests for Monetization Services

**Category**: Testing

**Description**:
Write comprehensive unit tests for all monetization backend services. This includes:
- Unit tests for PaymentManager
- Unit tests for PremiumService
- Unit tests for AdManager
- Unit tests for LocalStorage
- Mock payment provider and ad provider APIs
- Test error cases and edge cases

**Expected Output**:
- Unit test files for all monetization services
- Test coverage >80%
- All tests passing
- Mock implementations for external services

**Input Requirements**:
- All monetization services implemented
- Testing framework (Rust: `tokio-test`, `mockall`)

**Dependencies**:
- Task #7 (Implement PaymentManager Service)
- Task #8 (Implement PremiumService)
- Task #9 (Implement AdManager Service)
- Task #6 (Create Local Storage Module)

**Acceptance Criteria**:
- [ ] Unit tests for all services
- [ ] Test coverage >80%
- [ ] All tests passing
- [ ] Error cases tested
- [ ] Edge cases tested
- [ ] Mocks for external services

**Complexity**: Medium
**Estimated Time**: 8-10 hours

**Implementation Notes**:
- Use `mockall` for mocking in Rust
- Test both success and failure paths
- Test concurrent operations if applicable
- Use test fixtures for common test data

---

### Task #24: Write Integration Tests

**Category**: Testing

**Description**:
Write integration tests for end-to-end monetization flows. This includes:
- Test free user deletion flow (ad → deletion)
- Test premium user deletion flow (no ad)
- Test premium purchase flow (payment → premium status)
- Test premium status persistence (app restart)
- Test error scenarios (ad failure, payment failure)
- Test restore purchase flow

**Expected Output**:
- Integration test files
- End-to-end test scenarios
- All integration tests passing
- Test documentation

**Input Requirements**:
- All monetization features implemented
- Test environment setup
- Test payment provider account
- Test ad provider account

**Dependencies**:
- All monetization tasks completed
- Task #23 (Write Unit Tests)

**Acceptance Criteria**:
- [ ] Integration tests for all major flows
- [ ] Tests use test payment provider
- [ ] Tests use test ad units
- [ ] All tests passing
- [ ] Error scenarios tested
- [ ] Tests are reliable and not flaky

**Complexity**: Medium-High
**Estimated Time**: 8-12 hours

**Implementation Notes**:
- Use test mode for payment and ad providers
- Test with real provider APIs (test mode)
- Clean up test data after tests
- Use test fixtures for consistent test data

---

## 📦 Phase 7: Documentation & Deployment

### Task #25: Update Documentation and Create Deployment Guide

**Category**: Documentation

**Description**:
Update project documentation and create deployment guide for monetization features. This includes:
- Update README with monetization features
- Document ad provider setup
- Document payment provider setup
- Create deployment checklist
- Document environment variables and configuration
- Create troubleshooting guide
- Update API documentation (if applicable)

**Expected Output**:
- Updated README.md
- Monetization setup guide
- Deployment checklist
- Configuration documentation
- Troubleshooting guide

**Input Requirements**:
- All monetization features completed
- Understanding of deployment process

**Dependencies**:
- All monetization tasks completed

**Acceptance Criteria**:
- [ ] README updated with monetization info
- [ ] Setup guides created
- [ ] Deployment checklist created
- [ ] Configuration documented
- [ ] Troubleshooting guide created
- [ ] All documentation clear and complete

**Complexity**: Low
**Estimated Time**: 4-6 hours

**Implementation Notes**:
- Include screenshots if helpful
- Document all required environment variables
- Include links to provider documentation
- Create quick start guide for developers

---

## Summary

### Critical Path
1. Research Tasks (1-3) → Foundation (4-6) → Backend (7-12) → Frontend (13-18) → Integration (19-22) → Testing (23-24) → Documentation (25)

### Parallel Work Opportunities
- Tasks #1 and #2 (Research tasks) can be done in parallel
- Tasks #4 and #5 (Setup tasks) can be done in parallel after research
- Task #6 (Local Storage) can be done in parallel with setup tasks
- Frontend tasks (13-18) can be partially done in parallel with backend tasks
- Task #23 (Unit Tests) can be written as services are completed

### Risk Items
- **Task #1-2 (Research)**: Critical - Wrong provider choice could require rework
- **Task #7 (PaymentManager)**: High complexity - Payment integration is complex
- **Task #13 (AdDisplayComponent)**: Medium complexity - Ad SDK integration can be tricky
- **Task #19 (Integration)**: Medium complexity - Ensuring all deletion points are covered
- **Task #24 (Integration Tests)**: High complexity - End-to-end testing with external services

### Definition of Done
- [ ] All tasks completed
- [ ] All tests passing (unit + integration)
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Tested in development environment
- [ ] Tested with test payment provider
- [ ] Tested with test ad units
- [ ] Ready for staging deployment
- [ ] Monetization features working end-to-end
- [ ] Premium status persists across app restarts
- [ ] Ads display correctly for free users
- [ ] Premium purchase flow works
- [ ] All deletion operations integrated with monetization

