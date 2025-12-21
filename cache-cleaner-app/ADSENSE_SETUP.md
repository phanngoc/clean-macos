# Google AdSense Setup Guide

This document provides step-by-step instructions for setting up Google AdSense integration in the Cache Cleaner application.

## Prerequisites

- Google account
- Access to Google AdSense
- Understanding of ad formats and monetization

## Step 1: Create AdSense Account

1. Visit [https://www.google.com/adsense/](https://www.google.com/adsense/)
2. Sign in with your Google account
3. Click "Get Started"
4. Enter your website/application details
5. Complete the account setup process

**Note**: AdSense approval can take 1-2 weeks. You can use test ad units during development.

## Step 2: Get Your Publisher ID

1. After account creation, go to AdSense dashboard
2. Navigate to **Account** → **Account information**
3. Copy your **Publisher ID** (format: `ca-pub-XXXXXXXXXX`)
4. Save this for later configuration

## Step 3: Create Ad Units

### For Rewarded Video Ads (15-second format):

1. Go to **Ads** → **By ad unit**
2. Click **Create new ad unit**
3. Select **Video ads** or **Display ads** (depending on your needs)
4. Configure:
   - **Ad format**: Rewarded video
   - **Ad size**: 320x480 (or responsive)
   - **Duration**: 15 seconds
5. Click **Create**
6. Copy the **Ad Unit ID** (format: `ca-app-pub-XXXXXXXXXX/XXXXXXXXXX`)

### For Display Ads:

1. Follow similar steps but select **Display ads**
2. Choose appropriate ad sizes
3. Copy the Ad Unit ID

## Step 4: Configure in Application

### Update HTML Configuration

1. Open `cache-cleaner-app/ui/index.html`
2. Locate the `ADSENSE_CONFIG` object (around line 330)
3. Update with your values:

```javascript
const ADSENSE_CONFIG = {
  publisherId: 'ca-pub-YOUR_ACTUAL_PUBLISHER_ID',
  adUnitId: 'ca-app-pub-YOUR_PUBLISHER_ID/YOUR_AD_UNIT_ID',
  testAdUnitId: 'ca-app-pub-3940256099942544/5224354917', // Google test ad - keep this
  environment: 'development', // Change to 'production' when ready
  adDurationSeconds: 15
};
```

### Update Script Tag

1. In the `<head>` section of `index.html`
2. Find the AdSense script tag
3. Replace `ca-pub-PLACEHOLDER` with your actual Publisher ID:

```html
<script async src="https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js?client=ca-pub-YOUR_PUBLISHER_ID"
   crossorigin="anonymous"></script>
```

## Step 5: Test Ad Loading

### Using Browser Console

1. Run the application: `cd src-tauri && cargo tauri dev`
2. Open browser DevTools (F12 or Cmd+Option+I)
3. Go to Console tab
4. Run: `testAdLoading()`
5. Verify that test ads load correctly

### Expected Behavior

- Ad container appears
- Ad loads from Google's servers
- Ad displays for configured duration (15 seconds)
- Console shows success message

## Step 6: Production Deployment

### Before Going Live

1. ✅ Ensure AdSense account is approved
2. ✅ Test ads work correctly
3. ✅ Verify ad placement doesn't break UI
4. ✅ Check privacy compliance (GDPR, CCPA)
5. ✅ Update environment to 'production'

### Switch to Production

1. Change `ADSENSE_CONFIG.environment` to `'production'`
2. Ensure `adUnitId` uses your production ad unit
3. Test one more time
4. Deploy

## Test Ad Units

Google provides test ad units for development:

- **Test Publisher ID**: `ca-pub-3940256099942544`
- **Test Ad Unit ID**: `ca-app-pub-3940256099942544/5224354917`

These are already configured in the code for development mode.

## Security Best Practices

1. **Never commit API keys** - Already handled by `.gitignore`
2. **Use environment variables** - Consider moving config to environment variables
3. **Validate ad responses** - Always verify ad completion before allowing actions
4. **Handle ad failures gracefully** - Don't block user actions if ads fail to load

## Troubleshooting

### Ads Not Loading

**Symptoms**: No ads appear, console shows errors

**Solutions**:
- Check Publisher ID and Ad Unit ID are correct
- Verify AdSense account is approved (for production)
- Check browser console for specific error messages
- Ensure CSP (Content Security Policy) allows AdSense scripts
- Try test ad units first

### CSP Errors

**Symptoms**: Console shows Content Security Policy violations

**Solutions**:
- Check `tauri.conf.json` CSP settings
- Add AdSense domains to allowed sources:
  ```json
  "security": {
    "csp": "default-src 'self'; script-src 'self' 'unsafe-inline' https://pagead2.googlesyndication.com; ..."
  }
  ```

### Ad Blockers

**Symptoms**: Ads don't load, no errors in console

**Solutions**:
- Users with ad blockers won't see ads (expected behavior)
- Consider showing alternative monetization message
- Don't block app functionality if ads are blocked

## Revenue Information

- **Revenue Share**: ~68% to publisher (varies by region)
- **Payment Threshold**: $100 minimum
- **Payment Schedule**: Monthly (around 21st of following month)
- **Payment Methods**: Bank transfer, check, or other methods

## Support Resources

- [AdSense Help Center](https://support.google.com/adsense/)
- [AdSense Policies](https://support.google.com/adsense/answer/48182)
- [AdSense API Documentation](https://developers.google.com/adsense)

## Next Steps

After completing this setup:

1. ✅ Task #4 complete - Ad Provider SDK and Account set up
2. ⏭️ Proceed to Task #5 - Set Up Payment Provider SDK and Account
3. ⏭️ Continue with backend integration (Task #7+)

