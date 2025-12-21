# Paddle Payment Provider Setup Guide

This guide walks you through setting up Paddle as the payment provider for the $15 premium license feature.

## Overview

Paddle is a payment provider specifically designed for software sales. It handles:
- Payment processing
- Tax calculation and collection
- License key generation
- Receipt validation
- Webhook notifications

## Prerequisites

- A valid email address for account creation
- Business information (for tax purposes)
- Bank account details (for payouts)

## Step 1: Create Paddle Developer Account

1. **Visit Paddle Signup**: Go to https://vendors.paddle.com/signup
2. **Create Account**:
   - Enter your email address
   - Choose a secure password
   - Verify your email address
3. **Complete Business Information**:
   - Business name
   - Business address
   - Tax identification number (if applicable)
   - Bank account details for payouts

## Step 2: Register Your Application/Product

1. **Navigate to Products**: In Paddle dashboard, go to "Products" section
2. **Create New Product**:
   - Click "Add Product"
   - Product Name: "Cache Cleaner Premium License"
   - Product Type: "One-time payment"
   - Price: **$15.00 USD**
   - Currency: USD
   - Description: "Premium license for Cache Cleaner app - removes ads and enables unlimited cache cleaning"
3. **Save Product ID**: 
   - After creating, note the Product ID (you'll need this for configuration)
   - Example format: `prod_xxxxxxxxxxxxx`

## Step 3: Obtain API Keys

1. **Navigate to Developer Tools**: In Paddle dashboard, go to "Developer Tools" → "Authentication"
2. **Generate API Key**:
   - Click "Create API Key"
   - Name it: "Cache Cleaner - Production" (or "Cache Cleaner - Test" for development)
   - Copy the API key immediately (you won't be able to see it again)
   - **Important**: Store this securely - never commit to version control

3. **Get Vendor ID**:
   - Go to "Account Settings" → "Account Details"
   - Find your "Vendor ID" (usually a numeric ID)
   - Copy this value

4. **Get Webhook Signing Key** (Optional but recommended):
   - Go to "Developer Tools" → "Webhooks"
   - Click "Add Webhook"
   - Set webhook URL (for production: your server endpoint)
   - Copy the "Signing Key" for webhook verification

## Step 4: Configure Test Mode

For development, you'll want to use Paddle's sandbox/test environment:

1. **Enable Test Mode**:
   - In Paddle dashboard, look for "Test Mode" or "Sandbox" toggle
   - Enable test mode for development
   - Test mode uses separate API keys and products

2. **Create Test Product**:
   - Create a duplicate product in test mode
   - Use the same $15 price
   - Note the test Product ID

3. **Get Test API Keys**:
   - Generate separate API keys for test mode
   - Test keys usually start with `test_` prefix

## Step 5: Configure Application

### Option A: Environment Variables (Recommended for Development)

Create a `.env` file in the `cache-cleaner-app/` directory:

```bash
# Paddle Configuration
PADDLE_API_KEY=your_api_key_here
PADDLE_VENDOR_ID=your_vendor_id_here
PADDLE_PRODUCT_ID=your_product_id_here
PADDLE_TEST_MODE=true
PADDLE_WEBHOOK_KEY=your_webhook_key_here  # Optional
```

**Important**: 
- Add `.env` to `.gitignore` (already done)
- Never commit `.env` files to version control
- Use test keys during development

### Option B: Configuration File (For Production)

The application will also read from `~/.cache-cleaner/paddle-config.json`:

```json
{
  "api_key": "your_api_key_here",
  "vendor_id": "your_vendor_id_here",
  "product_id": "your_product_id_here",
  "test_mode": false,
  "webhook_key": "your_webhook_key_here"
}
```

**Note**: Configuration file takes precedence over environment variables.

## Step 6: Verify Setup

1. **Test API Connection**:
   ```bash
   cd cache-cleaner-app/src-tauri
   cargo test payment::paddle::tests::test_paddle_client_creation
   ```

2. **Check Configuration Loading**:
   - Run the application
   - The payment module should load without errors
   - Check logs for any configuration warnings

3. **Test Payment Flow** (in test mode):
   - Use Paddle's test card numbers:
     - Success: `4242 4242 4242 4242`
     - Decline: `4000 0000 0000 0002`
   - Expiry: Any future date
   - CVC: Any 3 digits

## Step 7: Production Setup

When ready for production:

1. **Disable Test Mode**:
   - Set `PADDLE_TEST_MODE=false` in environment or config
   - Use production API keys
   - Use production Product ID

2. **Configure Webhooks** (Recommended):
   - Set up webhook endpoint on your server
   - Configure in Paddle dashboard
   - Verify webhook signatures

3. **Test Production Flow**:
   - Use real payment methods (small test transactions)
   - Verify receipt validation works
   - Check license key generation

## Security Best Practices

1. **Never commit API keys**:
   - ✅ Use `.env` files (already in `.gitignore`)
   - ✅ Use environment variables
   - ❌ Never hardcode in source code
   - ❌ Never commit to git

2. **Rotate keys regularly**:
   - Generate new API keys periodically
   - Revoke old keys when no longer needed

3. **Use separate keys for test/production**:
   - Never use production keys in development
   - Keep test and production environments separate

4. **Verify webhook signatures**:
   - Always verify webhook requests come from Paddle
   - Use the webhook signing key for verification

## Troubleshooting

### Issue: "API key invalid"
- **Solution**: Verify you're using the correct API key (test vs production)
- Check that the key hasn't been revoked in Paddle dashboard

### Issue: "Product not found"
- **Solution**: Verify Product ID matches the product in your Paddle account
- Ensure you're using test Product ID in test mode

### Issue: "Connection failed"
- **Solution**: Check internet connection
- Verify API endpoint URLs are correct
- Check firewall settings

### Issue: "Test mode not working"
- **Solution**: Ensure `PADDLE_TEST_MODE=true` is set
- Verify you're using test API keys and test Product ID

## Next Steps

After completing this setup:

1. ✅ Task #5 acceptance criteria met
2. Proceed to Task #6: Implement Payment Verification Logic
3. Proceed to Task #7: Create Payment UI Components

## References

- [Paddle Developer Documentation](https://developer.paddle.com)
- [Paddle API Reference](https://developer.paddle.com/api-reference)
- [Paddle Webhooks Guide](https://developer.paddle.com/webhook-reference)
- [Paddle Test Mode Guide](https://developer.paddle.com/concepts/test-mode)

## Support

- Paddle Support: https://paddle.com/support
- Paddle Developer Community: https://paddle.com/community

