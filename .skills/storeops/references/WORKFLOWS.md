# StoreOps Workflows

Step-by-step guides for common app store tasks. Each workflow lists the exact commands in order.

---

## 1. First-Time Setup

```bash
# Option A: Login with flags
storeops auth login --store apple \
  --key-id KEY_ID --issuer-id ISSUER_ID --key-path /path/to/key.p8 \
  --name apple-prod

storeops auth login --store google \
  --service-account /path/to/sa.json \
  --name google-prod

# Option B: From environment variables
export STOREOPS_APPLE_KEY_ID=...
export STOREOPS_APPLE_ISSUER_ID=...
export STOREOPS_APPLE_KEY_PATH=...
export STOREOPS_GOOGLE_SERVICE_ACCOUNT=...
storeops auth init

# Verify
storeops auth status
```

---

## 2. Release a New iOS Version (End-to-End)

**Goal**: Create a version, set metadata, attach a build, and submit.

```bash
# Step 1: Find the app
APP_ID=$(storeops apple apps list | jq -r '.data[] | select(.attributes.bundleId=="com.example.app") | .id')

# Step 2: Find the latest processed build
BUILD_ID=$(storeops apple builds list --app-id "$APP_ID" --limit 1 | jq -r '.data[0].id')

# Step 3: Create the version
VER_ID=$(storeops apple versions create --app-id "$APP_ID" --version "2.0.0" --platform ios | jq -r '.data.id')

# Step 4: Update release notes for each locale
storeops apple metadata localizations update --version-id "$VER_ID" --locale en-US \
  --whats-new "New features and bug fixes."
storeops apple metadata localizations update --version-id "$VER_ID" --locale ja \
  --whats-new "新機能とバグ修正。"

# Step 5: Attach the build
storeops apple versions update --version-id "$VER_ID" --build-id "$BUILD_ID"

# Step 6: Submit for review
storeops apple submit "$APP_ID" --version "2.0.0"

# Step 7: Verify submission state
storeops apple versions list --app-id "$APP_ID" --limit 1 | jq '.data[0].attributes.appStoreState'
```

---

## 3. Release to Google Play with Staged Rollout

```bash
# Step 1: Upload the bundle
VCODE=$(storeops google builds upload --app-id com.example.app --file app-release.aab | jq -r '.data.versionCode')

# Step 2: Update the production track with 10% rollout
storeops google tracks update --app-id com.example.app --track production \
  --version-code "$VCODE" --rollout-fraction 0.1

# Step 3: Add release notes
storeops google tracks update --app-id com.example.app --track production \
  --release-notes '[{"language":"en-US","text":"Bug fixes."}]'

# Step 4: Commit the edit
storeops google submit --app-id com.example.app

# Later: increase rollout to 50%, then 100%
storeops google tracks update --app-id com.example.app --track production \
  --version-code "$VCODE" --rollout-fraction 0.5
storeops google submit --app-id com.example.app

storeops google tracks update --app-id com.example.app --track production \
  --version-code "$VCODE" --rollout-fraction 1.0
storeops google submit --app-id com.example.app
```

---

## 4. Update Store Listings Across All Locales

**Apple**:
```bash
VER_ID="..."

# Get all existing localizations
LOCALES=$(storeops apple metadata localizations list --version-id "$VER_ID" --paginate | jq -r '.data[].attributes.locale')

# Update each
for LOCALE in $LOCALES; do
  storeops apple metadata localizations update --version-id "$VER_ID" --locale "$LOCALE" \
    --whats-new "$(cat "release_notes/${LOCALE}.txt")"
done
```

**Google**:
```bash
APP=com.example.app
for LOCALE in en-US ja de-DE fr-FR pt-BR; do
  storeops google listings update --app-id "$APP" --locale "$LOCALE" \
    --title "$(jq -r ".${LOCALE}.title" listings.json)" \
    --short-description "$(jq -r ".${LOCALE}.short" listings.json)" \
    --full-description "$(jq -r ".${LOCALE}.full" listings.json)"
done
storeops google submit --app-id "$APP"
```

---

## 5. Upload Screenshots for All Device Types (Apple)

```bash
VER_ID="..."
LOCALE="en-US"
DISPLAYS=("APP_IPHONE_67" "APP_IPHONE_65" "APP_IPAD_PRO_129")

for DISPLAY in "${DISPLAYS[@]}"; do
  # Create the set
  SET_ID=$(storeops apple screenshots sets create \
    --version-id "$VER_ID" --locale "$LOCALE" --display-type "$DISPLAY" \
    | jq -r '.data.id')

  # Upload all images in the folder
  for FILE in "screenshots/${DISPLAY}"/*.png; do
    storeops apple screenshots images upload --set-id "$SET_ID" --file "$FILE"
  done
done
```

---

## 6. Upload Screenshots (Google)

```bash
APP=com.example.app
LOCALE=en-US

# Clear existing
storeops google images delete-all --app-id "$APP" --locale "$LOCALE" --image-type phoneScreenshots

# Upload new
for FILE in screenshots/phone/*.png; do
  storeops google images upload --app-id "$APP" --locale "$LOCALE" \
    --image-type phoneScreenshots --file "$FILE"
done

storeops google submit --app-id "$APP"
```

---

## 7. Manage TestFlight Beta

```bash
APP_ID="..."

# Create a group
GROUP_ID=$(storeops apple testflight groups create --app-id "$APP_ID" --name "QA Team" | jq -r '.data.id')

# Add testers from a list
while IFS=, read -r EMAIL FIRST LAST; do
  storeops apple testflight testers add --group-id "$GROUP_ID" \
    --email "$EMAIL" --first-name "$FIRST" --last-name "$LAST"
done < testers.csv

# List all testers to confirm
storeops apple testflight testers list --group-id "$GROUP_ID" --paginate
```

---

## 8. Google Play Internal Testing

```bash
APP=com.example.app

# Upload build
storeops google builds upload --app-id "$APP" --file app-debug.aab

# Assign to internal track
storeops google tracks update --app-id "$APP" --track internal --version-code 99

# Add testers
storeops google testers add --app-id "$APP" --track internal --email qa@example.com

storeops google submit --app-id "$APP"
```

---

## 9. Create an In-App Purchase (Apple)

```bash
APP_ID="..."

# Create the IAP
IAP_ID=$(storeops apple iap create --app-id "$APP_ID" \
  --product-id com.example.gems500 --type consumable \
  --reference-name "500 Gems" | jq -r '.data.id')

# Localize
storeops apple iap localizations create --iap-id "$IAP_ID" --locale en-US \
  --display-name "500 Gems" --description "Purchase 500 gems"
storeops apple iap localizations create --iap-id "$IAP_ID" --locale ja \
  --display-name "500ジェム" --description "500ジェムを購入"

# Set price
PRICE=$(storeops apple pricing points --app-id "$APP_ID" | jq -r '.data[] | select(.attributes.customerPrice=="4.99") | .id')
storeops apple iap prices set --iap-id "$IAP_ID" --price-point "$PRICE"

# Submit for review
storeops apple iap submit --iap-id "$IAP_ID"
```

---

## 10. Create a Subscription (Apple)

```bash
APP_ID="..."

# Create or get subscription group
GRP_ID=$(storeops apple subscriptions groups create --app-id "$APP_ID" \
  --reference-name "Pro Plans" | jq -r '.data.id')

# Create subscription item
SUB_ID=$(storeops apple subscriptions items create --group-id "$GRP_ID" \
  --product-id com.example.pro_monthly --reference-name "Pro Monthly" \
  --duration P1M | jq -r '.data.id')

# Localize
storeops apple subscriptions localizations create --subscription-id "$SUB_ID" \
  --locale en-US --display-name "Pro Monthly" --description "Unlock all features"

# Set pricing
storeops apple subscriptions prices set --subscription-id "$SUB_ID" \
  --price-point PRICE_ID

# Create an introductory offer (1 week free)
storeops apple subscriptions offers create --subscription-id "$SUB_ID" \
  --type introductory --duration P1W --mode free
```

---

## 11. Respond to Recent Negative Reviews

```bash
# Apple
REVIEWS=$(storeops apple reviews list --app-id "$APP_ID" --limit 50 --rating 1,2)
echo "$REVIEWS" | jq -r '.data[] | "\(.id): \(.attributes.title) - \(.attributes.body)"'

# Respond to each
for ID in $(echo "$REVIEWS" | jq -r '.data[].id'); do
  storeops apple reviews respond --review-id "$ID" \
    --response "We're sorry to hear about your experience. Please contact support@example.com."
done

# Google
REVIEWS=$(storeops google reviews list --app-id com.example.app)
for ID in $(echo "$REVIEWS" | jq -r '.data[].reviewId'); do
  storeops google reviews reply --review-id "$ID" \
    --reply "Thank you for the feedback. We're working on improvements."
done
```

---

## 12. Phased Release Management (Apple)

```bash
VER_ID="..."

# Start phased release (after approval)
storeops apple phased-release create --version-id "$VER_ID"

# Check progress
storeops apple phased-release get --version-id "$VER_ID"

# Pause if issues arise
storeops apple phased-release update --version-id "$VER_ID" --state PAUSE

# Resume
storeops apple phased-release update --version-id "$VER_ID" --state ACTIVE

# Release to everyone immediately
storeops apple phased-release update --version-id "$VER_ID" --state COMPLETE
```

---

## 13. Set Age Rating (Apple)

```bash
storeops apple age-rating get --app-id "$APP_ID"

storeops apple age-rating update --app-id "$APP_ID" \
  --alcohol-tobacco-drugs NONE \
  --gambling false \
  --violence INFREQUENT \
  --sexual-content NONE \
  --mature-content false
```

---

## 14. Cross-Platform Release (Apple + Google)

Run both stores in parallel when releasing the same version:

```bash
APP_APPLE="123456789"
APP_GOOGLE="com.example.app"

# Apple side
BUILD_ID=$(storeops apple builds list --app-id "$APP_APPLE" --limit 1 | jq -r '.data[0].id')
VER_ID=$(storeops apple versions create --app-id "$APP_APPLE" --version "3.0.0" --platform ios | jq -r '.data.id')
storeops apple metadata localizations update --version-id "$VER_ID" --locale en-US --whats-new "Version 3.0!"
storeops apple versions update --version-id "$VER_ID" --build-id "$BUILD_ID"
storeops apple submit "$APP_APPLE" --version "3.0.0"

# Google side
storeops google builds upload --app-id "$APP_GOOGLE" --file app-release.aab
VCODE=$(storeops google builds list --app-id "$APP_GOOGLE" --limit 1 | jq -r '.data[0].versionCode')
storeops google tracks update --app-id "$APP_GOOGLE" --track production \
  --version-code "$VCODE" --rollout-fraction 0.1
storeops google listings update --app-id "$APP_GOOGLE" --locale en-US \
  --short-description "Version 3.0 is here!"
storeops google submit --app-id "$APP_GOOGLE"
```

---

## 15. Audit Current State

Quick commands to understand the current state of an app:

```bash
# Apple: what's live, what's pending?
storeops apple versions list --app-id "$APP_ID" | jq '.data[] | {version: .attributes.versionString, state: .attributes.appStoreState}'

# Apple: latest build status
storeops apple builds list --app-id "$APP_ID" --limit 3 --output table

# Google: current track status
storeops google tracks list --app-id com.example.app --output table

# Review counts
storeops apple reviews list --app-id "$APP_ID" --limit 1 | jq '.meta.total'
storeops google reviews list --app-id com.example.app --limit 1 | jq '.meta.total'
```
