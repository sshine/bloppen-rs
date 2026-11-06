# AndroidManifest analysis

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Decoded with:** apktool 3.0.3 (`apktool d -f -o re/decompiled/apktool apk/extracted/base.apk`)

The decoded manifest is `re/decompiled/apktool/AndroidManifest.xml`; the APK
tool metadata is `apktool.yml`. Values that behave like credentials or app
identity secrets (Facebook app ID and client token, Google API keys, OAuth web
client ID, Firebase sender ID, storage bucket) are copied to
`secrets/manifest-strings.xml`, which is gitignored.

## Identity and SDK levels

| Field | Value |
| --- | --- |
| Package | `dk.boerneloppen.boerneloppen` |
| Version | 1.0.4 (versionCode 23) |
| minSdk / targetSdk / compileSdk | 23 / 35 / 35 |
| Application class | `dk.boerneloppen.boerneloppenapp.app.BoerneloppenApp` |
| Theme | `Theme.RowebBaseTemplateAndroidApp` |
| Launcher activity | `dk.boerneloppen.boerneloppenapp.screens.main.MainActivity` |
| Required split types | `base__density` |

The theme name and package structure suggest the app was built by Roweb from a
template, with the app code under `dk.boerneloppen.boerneloppenapp`.

## Permissions

| Permission | Likely purpose |
| --- | --- |
| `INTERNET` | API and image traffic |
| `ACCESS_NETWORK_STATE` | Connectivity checks |
| `POST_NOTIFICATIONS` | Firebase push notifications |
| `WAKE_LOCK` | Firebase messaging and analytics |
| `CAMERA` | QR/barcode scanning, stand or receipt features (confirm in Phase 3) |
| `READ_EXTERNAL_STORAGE` (maxSdk 32) | Selecting or reading images |
| `WRITE_EXTERNAL_STORAGE` (maxSdk 28) | Legacy image storage |
| `USE_BIOMETRIC`, `USE_FINGERPRINT` | Biometric login/unlock |
| `com.google.android.gms.permission.AD_ID` | Facebook/Google advertising ID |
| `com.google.android.finsky.permission.BIND_GET_INSTALL_REFERRER_SERVICE` | Install attribution |
| `com.google.android.c2dm.permission.RECEIVE` | Firebase Cloud Messaging |
| `dk.boerneloppen.boerneloppen.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION` | Signature-level guard for dynamic receivers |

There is a `<queries>` entry for `com.facebook.katana`, so Facebook Login can
detect the native Facebook app. There is no `QUERY_ALL_PACKAGES` and no broad
package visibility request.

## Application flags

| Flag | Value | Note |
| --- | --- | --- |
| `allowBackup` | `false` | Good: no ADB/cloud backup of app data |
| `fullBackupContent` | `@xml/backup_rules` | Empty `<full-backup-content />` |
| `dataExtractionRules` | `@xml/data_extraction_rules` | Empty `<cloud-backup />` |
| `extractNativeLibs` | `false` | Consistent with the absence of native libraries |
| `requestLegacyExternalStorage` | `true` | Legacy storage flag; ineffective on modern Android |
| `usesCleartextTraffic` | `true` | **HTTP is permitted app-wide** |
| `networkSecurityConfig` | `@xml/network_security_config` | See below |
| `supportsRtl` | `true` | |
| `windowSoftInputMode` | `adjustResize` | |

## Network security

`res/xml/network_security_config.xml`:

```xml
<network-security-config>
    <base-config cleartextTrafficPermitted="true">
        <trust-anchors>
            <certificates src="@raw/isrg_root_x2" />
            <certificates src="@raw/isrg_root_x1" />
            <certificates src="system" />
        </trust-anchors>
    </base-config>
</network-security-config>
```

Findings:

- **Cleartext HTTP is allowed for every domain.** There is no
  `<domain-config>` and no per-domain exception, so the API may be reachable
  over plain HTTP. Phase 3/4 must determine whether the real base URL is HTTP
  or HTTPS, and whether tokens travel over cleartext.
- **No certificate pinning.** The config adds the ISRG (Let's Encrypt) roots to
  the system trust store; it does not restrict trust to a leaf or pin set.
- The bundled ISRG roots are probably for old Android versions that predate
  Let's Encrypt in the system store. They do not make pinning stronger.

## Components

### Activities

| Component | Exported | Intent filters / notes |
| --- | --- | --- |
| `...screens.main.MainActivity` | `true` | `MAIN` + `LAUNCHER`; `singleTask`; splash theme; the only app activity |
| `com.facebook.FacebookActivity` | `false` | Facebook SDK login UI |
| `com.facebook.CustomTabActivity` | `true` | `VIEW` + `BROWSABLE` for `@string/fb_login_protocol_scheme` and `fbconnect://cct.dk.boerneloppen.boerneloppen` |
| `com.facebook.CustomTabMainActivity` | `false` | Facebook SDK helper |
| `com.google.android.gms.common.api.GoogleApiActivity` | `false` | Google Play services consent/error UI |

### Services

| Component | Exported | Notes |
| --- | --- | --- |
| `...messaging.BoerneloppenFirebaseMessagingService` | `true` | Handles `com.google.firebase.MESSAGING_EVENT` |
| `com.google.firebase.messaging.FirebaseMessagingService` | `false` | SDK base service |
| `com.google.firebase.components.ComponentDiscoveryService` | `false` | Registers Analytics, Messaging, Crashlytics, Installations, Datatransport |
| `com.google.android.gms.measurement.AppMeasurementService` | `false` | Analytics |
| `com.google.android.gms.measurement.AppMeasurementJobService` | `false` | Analytics job, guarded by `BIND_JOB_SERVICE` |
| `com.google.android.datatransport.runtime.backends.TransportBackendDiscovery` | `false` | Datatransport |
| `com.google.android.datatransport.runtime.scheduling.jobscheduling.JobInfoSchedulerService` | `false` | Datatransport job |

### Receivers

| Component | Exported | Notes |
| --- | --- | --- |
| `com.facebook.CurrentAccessTokenExpirationBroadcastReceiver` | `false` | Facebook token expiry |
| `com.facebook.AuthenticationTokenManager$CurrentAuthenticationTokenChangedBroadcastReceiver` | `false` | Facebook auth token changes |
| `com.google.android.gms.measurement.AppMeasurementReceiver` | `false` | Analytics |
| `com.google.firebase.iid.FirebaseInstanceIdReceiver` | `true` | C2DM receive, guarded by `com.google.android.c2dm.permission.SEND` |
| `androidx.profileinstaller.ProfileInstallReceiver` | `true` | Guarded by `android.permission.DUMP` |
| `com.google.android.datatransport.runtime.scheduling.jobscheduling.AlarmManagerSchedulerBroadcastReceiver` | `false` | Datatransport |

### Providers

| Component | Exported | Notes |
| --- | --- | --- |
| `androidx.core.content.FileProvider` | `false` | Authority `dk.boerneloppen.boerneloppen.fileprovider`; grants URI permissions |
| `com.facebook.internal.FacebookInitProvider` | `false` | Facebook SDK initialization |
| `com.google.firebase.provider.FirebaseInitProvider` | `false` | Firebase initialization, `initOrder=100` |
| `androidx.startup.InitializationProvider` | `false` | Emoji2, ProcessLifecycle, ProfileInstaller initializers |

## Deep links

The only exported deep links are Facebook Login callbacks:

| URI | Owner |
| --- | --- |
| `fbconnect://cct.dk.boerneloppen.boerneloppen` | `com.facebook.CustomTabActivity` |
| `@string/fb_login_protocol_scheme` (value `fb1234`) | `com.facebook.CustomTabActivity` |

There are no `https://` App Links, no custom `boerneloppen://` scheme, and no
`VIEW` intent filter on `MainActivity`. The app does not appear to accept
external deep links into loyalty or stand screens; navigation happens inside
Compose after the launcher activity starts.

## Third-party SDK inventory

| SDK | Evidence |
| --- | --- |
| Facebook Login | `FacebookActivity`, `CustomTabActivity`, `FacebookInitProvider`, consent strings |
| Firebase Cloud Messaging | `BoerneloppenFirebaseMessagingService`, `FirebaseMessagingService`, `FirebaseInstanceIdReceiver` |
| Firebase Analytics | `AppMeasurement*`, `FirebaseAnalyticsKtxRegistrar`, `google_app_id` |
| Firebase Crashlytics | `CrashlyticsRegistrar`, `FirebaseCrashlyticsKtxRegistrar` |
| Firebase Installations / Datatransport | registrars, `TransportBackendDiscovery` |
| Google Play services | `GoogleApiActivity`, `google_play_services_version`, `default_web_client_id` |
| AndroidX Compose + Navigation Compose | `META-INF/androidx.compose*` and `androidx.navigation*` version files |
| AndroidX Biometric | `USE_BIOMETRIC` permission and `androidx.biometric` version file |
| AndroidX Splashscreen / ProfileInstaller / Startup / Emoji2 | providers and version files |
| Retrofit2, OkHttp, Okio | DEX descriptors (see file census) |
| Coil | DEX descriptors (image loading) |
| Timber | DEX descriptors (logging) |
| Kotlin coroutines | DEX descriptors and `kotlinx_coroutines*` version files |

`facebook_app_id`, `facebook_client_token`, `default_web_client_id`,
`google_api_key`, `google_crash_reporting_api_key`, `gcm_defaultSenderId`,
`project_id`, and `google_storage_bucket` are present in `strings.xml`. Exact
values are in `secrets/manifest-strings.xml`. The Firebase project name is
`boerneloppen-1b02a`; the declared user agent is `BoerneloppenApp`.

## Likely entry points for Phase 3

1. **Application startup:** `BoerneloppenApp` and
   `MainActivity` initialize Firebase, Facebook, dependency injection, and the
   Compose navigation graph.
2. **Login:** Facebook Login is wired through the SDK callbacks. The presence
   of `default_web_client_id` also suggests Google Sign-In or Firebase
   Authentication, but that needs confirmation in the auth classes. Biometric
   permissions suggest an unlock step rather than first-factor login.
3. **Loyalty and stand features:** there is no separate activity for either.
   They are Compose destinations inside `MainActivity`. The navigation graph
   and screen composables are the place to look for the loyalty card, points,
   stand booking, and QR/camera flows.
4. **Push:** `BoerneloppenFirebaseMessagingService` is the only custom service;
   it likely routes notification payloads into the same navigation graph.

## Security observations for later phases

- Cleartext is allowed globally; confirm the actual base URL scheme before
  assuming TLS.
- No certificate pinning in the network security config. Check OkHttp for a
  programmatic `CertificatePinner`.
- `requestLegacyExternalStorage=true` is stale for a targetSdk 35 app.
- `allowBackup=false` and empty backup rules are good.
- Only the launcher activity, Facebook callbacks, and Firebase entry points are
  exported. The attack surface is small and mostly SDK-owned.
