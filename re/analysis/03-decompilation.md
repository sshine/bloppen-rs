# Decompilation results

**Build:** Børneloppen 1.0.4 (versionCode 23), `dk.boerneloppen.boerneloppen`
**Base APK SHA-256:** `54a2a8062319c3d9c31f23f11a31541eec1a28ba82e8186e1c374e41fce55bb3`

## Command

```sh
./re/decompile.sh apk/extracted/base.apk re/decompiled
```

The script runs both decompilers fresh and records their output under
`re/decompiled/`:

| Output | Tool | Purpose |
| --- | --- | --- |
| `re/decompiled/jadx/` | jadx 1.5.6 | Java-like sources for Phase 3 |
| `re/decompiled/apktool/` | apktool 3.0.3 | Decoded manifest, resources, assets, smali |
| `re/decompiled/jadx.log` | jadx | Per-pass progress and errors |
| `re/decompiled/apktool.log` | apktool | Resource warnings |

`JADX_OPTS` and `JAVA_OPTS` default to `-Xmx4g`. The whole run took **51 s**
for jadx and **11 s** for apktool on this machine.

## Output size

| Output | Size | Files |
| --- | ---: | ---: |
| `re/decompiled/jadx` | 210 MiB | 19,734 `.java` |
| `re/decompiled/apktool` | 408 MiB | 30,749 `.smali`, plus resources and assets |

The generated trees and logs are gitignored.

## jadx errors

jadx exits with status 3 and logs:

```text
ERROR - finished with errors, count: 228
```

That is 228 method/class errors out of 16,993 processed classes. The output is
still usable; the failures are concentrated in third-party Kotlin coroutine,
Coil, and Okio code.

| Error category | Occurrences |
| --- | ---: |
| `JadxOverflowException` in `RegionMakerVisitor` | 113 |
| `JadxRuntimeException` in `ModVisitor` | 17 |
| `JadxRuntimeException` in `ConstructorVisitor` | 8 |
| `JadxRuntimeException` in `FinishTypeInference` | 5 |
| Method generation / code generation errors | 7 |
| `UnsupportedOperationException` in `RegionMakerVisitor` | 2 |
| `StackOverflowError`, type-fix, and other region errors | 4 |

### App-package failures

Only **three files under `dk/boerneloppen/`** contain jadx error comments:

- `dk/boerneloppen/boerneloppenapp/screens/homescreen/HomeScreenKt.java`
- `dk/boerneloppen/boerneloppenapp/screens/mainscreen/MainScreenKt.java`
- `dk/boerneloppen/boerneloppenapp/screens/menuitemscreen/MenuItemScreenKt.java`

All three are Jetpack Compose screen files, where compiler-generated lambdas
and state objects are the hardest input for a decompiler. The API, domain,
model, and login classes decompiled without error comments. For any specific
Compose method that matters, the corresponding smali under
`re/decompiled/apktool/smali_classes*/` is the fallback, and jadx can be rerun
with `--show-bad-code` or `--single-class`.

## apktool warnings

apktool logged **80 unresolved resource references**. They are all
`ResReference` entries for the app package (`0x7f...`), typically generated
Compose or Material resources that apktool cannot name without the full
resource table. They did not stop the manifest, resources, or smali from being
decoded.

## Decompiled app structure

The app code is a single Kotlin/Compose application under
`dk.boerneloppen.boerneloppenapp`. Java-file counts by package:

| Package | Java files | Likely role |
| --- | ---: | --- |
| `screens/homescreen` | 32 | Home/loyalty UI |
| `screens/loginscreen` | 24 | Login UI and view model |
| `screens/mainscreen` | 14 | Main shell / navigation host |
| `domain` | 12 | Use cases |
| `screens/menuitemscreen` | 9 | Menu/receipt/stand UI |
| `ui` + `ui/theme` | 12 | Compose design system |
| `entities` | 7 | Alert and message models |
| `api` (all subpackages) | 3 + 4 + 4 + 3 + 3 + 1 | Retrofit client, interceptors, errors, models |
| `util/store` | 4 | Local persistence |
| `util/keystore` + models | 4 | Secure storage / biometrics |
| `messaging` | 3 | Firebase messaging |
| `navigation` | 2 | Compose navigation |
| `data` | 2 | Repositories or data sources |
| `constants`, `extensions`, `util` | 8 | Support code |
| `app` | 1 | `BoerneloppenApp` application class |

The following API-side classes decompiled cleanly and are the natural starting
points for Phase 3:

- `api/BoerneloppenAPI.java` and `api/BoerneloppenAPIImpl.java`
- `api/RetrofitBoerneloppenApi.java`
- `api/interceptor/NetworkInterceptor.java`,
  `RetrofitErrorInterceptor.java`, `AcceptTypeHeaderChangerInterceptor.java`
- `api/models/LoginTokenModel.java`, `ResponseModel.java`,
  `ErrorResponseModel.java`
- `api/exceptions/UnauthorizedException.java`, `ForbiddenException.java`,
  `NetworkException.java`, `GenericApiError.java`
- `domain/RequestLoginTokenUseCase.java`, `GetLoginTokenUseCase.java`,
  `GetLoginStatusUseCase.java`, `SaveLoginTokenUseCase.java`
- `domain/GetVmcmsCookieUseCase.java`, `VmcmsCookieUseCase.java`
- `domain/GetFirebaseTokenUseCase.java`, `UploadFirebaseTokenUseCase.java`
- `domain/FacebookConsentUseCase.java`, `LogoutUseCase.java`

The presence of a `VmcmsCookie` use case and a `BoerneloppenAPI`/Retrofit
abstraction is the main Phase 3 lead; the exact base URL and auth exchange
belong in the next phase, not here.

## Known gaps

- Compose-generated code is the only app code with jadx errors. Smali is
  available for those classes.
- Kotlin coroutine internals and Coil/Okio have partial Java output. They are
  library code and not expected to matter for the API client.
- apktool's unresolved resource references mean some resource names are
  numeric. The manifest and resource files needed for analysis are readable.
- No native libraries exist, so there is no `radare2`/`readelf` work to do
  (see [`04-native-libs.md`](04-native-libs.md)).

## Reproducing

```sh
nix develop
./re/decompile.sh apk/extracted/base.apk re/decompiled
grep -c 'JADX ERROR' re/decompiled/jadx.log
find re/decompiled/jadx -name '*.java' | wc -l
```
