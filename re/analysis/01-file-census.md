# File census: `base.apk`

- Size: 46.9 MiB (49,180,237 bytes)
- SHA-256: `54a2a8062319c3d9c31f23f11a31541eec1a28ba82e8186e1c374e41fce55bb3`
- Entries: 855 files

## Top-level layout

| Top-level path | Files |
| --- | ---: |
| `res` | 704 |
| `META-INF` | 94 |
| `(root)` | 43 |
| `kotlin` | 7 |
| `assets` | 3 |
| `junit` | 2 |
| `okhttp3` | 2 |

## File types

| Extension | Files | Uncompressed size |
| --- | ---: | ---: |
| `.xml` | 664 | 606.0 KiB |
| `.version` | 87 | 2.2 KiB |
| `.properties` | 34 | 2.7 KiB |
| `.png` | 30 | 195.8 KiB |
| `.ttf` | 9 | 1.3 MiB |
| `.kotlin_builtins` | 7 | 29.2 KiB |
| `.dex` | 4 | 45.1 MiB |
| `.gif` | 2 | 1.8 KiB |
| `.(none)` | 2 | 250 B |
| `.der` | 2 | 1.9 KiB |
| `.bin` | 1 | 1.7 KiB |
| `.txt` | 1 | 11.1 KiB |
| `.coroutineexceptionhandler` | 1 | 54 B |
| `.maindispatcherfactory` | 1 | 52 B |
| `.textproto` | 1 | 120 B |
| `.prof` | 1 | 8.8 KiB |
| `.profm` | 1 | 496 B |
| `.html` | 1 | 657 B |
| `.json` | 1 | 625 B |
| `.gz` | 1 | 36.8 KiB |

## Largest entries

| Size | Compressed | Entry |
| ---: | ---: | --- |
| 28.5 MiB | 28.5 MiB | `classes.dex` |
| 8.5 MiB | 8.5 MiB | `classes3.dex` |
| 8.0 MiB | 8.0 MiB | `classes4.dex` |
| 541.0 KiB | 541.0 KiB | `resources.arsc` |
| 154.2 KiB | 63.1 KiB | `res/font/poppins_thin.ttf` |
| 154.0 KiB | 67.2 KiB | `res/font/poppins_extralight.ttf` |
| 152.5 KiB | 67.2 KiB | `res/font/poppins_light.ttf` |
| 151.0 KiB | 67.2 KiB | `res/font/poppins_regular.ttf` |
| 149.3 KiB | 66.5 KiB | `res/font/poppins_medium.ttf` |
| 148.0 KiB | 67.0 KiB | `res/font/poppins_semibold.ttf` |
| 146.8 KiB | 66.4 KiB | `res/font/poppins_bold.ttf` |
| 145.6 KiB | 66.1 KiB | `res/font/poppins_extrabold.ttf` |
| 144.2 KiB | 64.7 KiB | `res/font/poppins_black.ttf` |
| 97.2 KiB | 35.8 KiB | `META-INF/BNDLTOOL.SF` |
| 97.1 KiB | 33.2 KiB | `META-INF/MANIFEST.MF` |
| 36.8 KiB | 36.8 KiB | `okhttp3/internal/publicsuffix/publicsuffixes.gz` |
| 20.7 KiB | 4.5 KiB | `AndroidManifest.xml` |
| 18.2 KiB | 5.1 KiB | `kotlin/kotlin.kotlin_builtins` |
| 17.4 KiB | 17.4 KiB | `res/mipmap-xxxhdpi-v4/ic_launcher_foreground.png` |
| 15.9 KiB | 15.9 KiB | `res/mipmap-xxxhdpi-v4/ic_launcher_round.png` |

## DEX files

| File | Size |
| --- | ---: |
| `classes.dex` | 28.5 MiB |
| `classes2.dex` | 1.9 KiB |
| `classes3.dex` | 8.5 MiB |
| `classes4.dex` | 8.0 MiB |

## Native libraries

No `lib/<abi>/*.so` entries. Native-code inspection is not required.

## Assets and resources

- `assets/`: 3 files, 9.9 KiB uncompressed
- `res/`: 704 files, 2.1 MiB uncompressed
- `resources.arsc`: present
  - `assets/dexopt/baseline.prof`
  - `assets/dexopt/baseline.profm`
  - `assets/error.html`

## Framework identification

| Framework | Filename evidence | DEX evidence |
| --- | --- | --- |
| Flutter | - | - |
| React Native | - | - |
| Xamarin / .NET | - | - |
| Capacitor | - | - |
| Cordova | - | - |
| Ionic | - | - |

**Detected:** no Flutter, React Native, Xamarin/.NET, Capacitor, or Cordova markers. This is a native Android app (Java/Kotlin).

## Managed-code check

No `assemblies/` directory, `*.dll`, or `*.exe` entries. The Xamarin/.NET path does not apply.

## Obfuscation indicators

- Type descriptors in DEX: 34,517
- DEX files that could not be parsed: 0
- Descriptors in the app namespace `dk/boerneloppen/`: 438
- Descriptors outside standard Java/Android/Kotlin packages: 3,711
- Descriptors whose class name is 1-2 characters: 151
- `mapping.txt` in APK: no
- No strong obfuscation signal: 0.44% of descriptors have 1-2 character class names.

Non-standard package roots (top 15):

- `com`: 1,420 descriptors
- `org`: 851 descriptors
- `dk`: 438 descriptors
- `coil`: 374 descriptors
- `okhttp3`: 346 descriptors
- `okio`: 120 descriptors
- `retrofit2`: 116 descriptors
- `junit`: 28 descriptors
- `io`: 8 descriptors
- `timber`: 6 descriptors
- `sun`: 3 descriptors
- `libcore`: 1 descriptors

## Summary

- Native Android app (Java/Kotlin); no Flutter, React Native, Xamarin/.NET, Capacitor, or Cordova markers.
- DEX: 4 files, 45.1 MiB uncompressed.
- Native libraries: none
- Managed assemblies: none
- Obfuscation: 0.44% short class names; mapping.txt absent.
- Notable non-standard package roots: `com`, `org`, `dk`, `coil`, `okhttp3`, `okio`, `retrofit2`, `junit`.
