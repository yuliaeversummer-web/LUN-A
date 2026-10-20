# Этап 7 — стабильность и измерения

## Аудит до исправлений

Проверены AudioService/MediaService/DiscordService, RAII callbacks/WinRT tokens,
bounded queues и deadlines, permissions/commands, Pinia, окна/tray, маскот,
обложки/иконки. Архитектура и семантика default eConsole сохранены.

| Приоритет | Результат |
|---|---|
| Critical | Подтверждённых проблем не найдено. |
| High | Нет независимого восстановления после потери Tauri event; ошибка первой подписки/чтения может оставить UI без синхронизации. |
| Medium | Оба скрытых WebView продолжают изменять Pinia с каждым timeline; `document.hidden=false` при нативном hide. Измерено 79 media mutations за 20,958 с в каждом скрытом окне. |
| Medium | Release CSP блокирует встроенный Vite data-font. Подтверждено console error в исходном release. |
| Low / риск | GSMTC повторно читает thumbnail при 5-секундной metadata сверке; frontend повторно не загружает неизменную обложку. Без отдельного профиля не меняем восстановление пропущенных metadata events. |
| Low / риск | Некоторые Core Audio session handles при ошибке чтения строки удерживаются до следующего изменения перечисления. Размер ограничен live IDs; горячее отключение требует отдельного hardware теста. |

Исходный release этапа 6 измерен до пересборки. Для CDP он перезапущен
с локальным `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` (порт 9223, localhost).
Это диагностическая остановка, не проверка штатного Exit. Музыка/Discord не
останавливались, реальные Toggle/volume/media-команды пока не отправлялись.

## QA inventory

- Реальные данные: default output + сессии всех активных render endpoints,
  точный Yandex desktop source, Discord unknown и сохранённые назначения.
- Окна: main visible/hidden/minimized, overlay visible/hidden/pin/autohide,
  повторный запуск, ограничение main-only команд из overlay.
- UI: обе страницы, 4 темы × 1100×720 / 850×560, overlay 360px;
  screenshots и проверка overflow, focus, reduced motion, состояния ошибок.
- Отдельный синтетический stress: длинные строки и большой список сессий;
  это только layout QA, команды к синтетическим ID не отправляются.
- Негативные сценарии: потерянное событие, старый ответ, таймаут чтения,
  поздняя подписка после dispose, запрещённая команда из overlay.
- Сравнение baseline/final и непрерывный soak, метрики JSONL каждые 30с.

## Исправления

- Общий `snapshotChannel` для Audio/Media/Discord/Desktop: одна подписка на
  состояние каждого сервиса в WebView, latest-only buffer скрытого окна,
  native show/hide/minimize, сверка cached DTO каждые 15с и при восстановлении.
- Отбрасываются устаревшие и повторные ревизии; при таймауте чтения за 5с UI
  выходит из loading в error. Чтение не блокирует неподтверждённая подписка.
  Позднее завершение listen после dispose вызывает unsubscribe.
- Восстановление с той же ревизией разрешено после ошибки. Дополнительный
  тест воспроизвёл поздний отказ listen уже после первого успешного read;
  исправлен failed marker. Артефакт воспроизведения:
  `output/stage-7/late-subscription-reproduction.log`.
- На restore пропущенные transient Next/Error не воспроизводятся заново.
  Источники реакций и таймеры маскота сохранены. Анимации скрытого WebView
  приостановлены CSS; backend workers продолжают работать.
- Двухоконный замер обнаружил ошибку новой обвязки: SDK listen по умолчанию
  использует Any, поэтому native visibility main кратко активировала overlay.
  Исправлено на `{target: windowHandle.label}`, добавлен epoch guard для CSS.
  `scoped-hidden-verified.log`: main visible 16 updates/4s, hidden overlay 0;
  overlay visible 15/4s, hidden main 0; оба hidden 0/0 при 15 backend revisions.
  Предыдущие прогоны сохранены; длительности и остановки описаны в итоговых измерениях ниже.
- Все локальные font/assets вынесены из data URI (`assetsInlineLimit: 0`),
  прежняя CSP не ослаблена. Исходные PNG не изменялись.
- Синтетическое имя устройства без пробелов растягивало внутренний список
  с 585 до 2588 CSS px. После `overflow-wrap:anywhere`: 585/585, 300 строк.

## Проверки кода и нативных функций

- Frontend regression: 11/11, `node --test tests/snapshotChannel.test.ts`.
  Покрыты старый ответ/дубликат, потеря события, hidden/restore, late listener,
  dispose, stalled listen, visibility race/error, timeout/same-revision recovery,
  late subscription failure. Новых frontend-зависимостей нет.
- Rust: 13/13; `cargo check`, typecheck, build и release/NSIS выполнены.
  Финальные пути логов/размер установщика добавляются ниже.
- Ctrl+D нативно открыл overlay при скрытом main; повторный Ctrl+D скрыл.
- Pin сохранил overlay после возврата фокуса main; unpin+autoHide скрыл его.
  Minimize оставил main свёрнутым; open_main восстановил его.
- Проверены cold `--autostart` с startInTray=false/true (main visible/hidden),
  повторный auto launch сохранил скрытие, ручной восстановил main. Регистрация
  автозапуска уже была включена пользователем; enabled не изменяли.
- Реальный конфликт: отдельный QA example временно зарегистрировал
  Ctrl+Alt+F8. Настройка LUNÉA отклонена, Ctrl+D остался зарегистрирован.
  Через 30с fixture снял регистрацию; input не отправлялся. F9 Discord тоже
  отклонён как конфликт. Повтор Ctrl+D не создаёт второй регистрации.
- После пользовательского «Выход» все три worker записали stopped, процесс=0.
  `debug-autostart-hidden.log`. WebView2 выдал unregister class 1412; завершение
  успешно. Последующие остановки при сборке — диагностические, не Exit-тесты.
- В реальном read-only состоянии: 8 аудиосессий, desktop source
  ru.yandex.desktop.music, Discord Stable, mute/deafen=null. Невалидная громкость,
  отсутствующая сессия и stale track token отклонены; реальные уровни не менялись.

## UI review

- Playwright CLI + persistent CDP, настоящий release WebView2; обе страницы,
  4 темы × 2 CSS viewport размера, 4 overlay, reduced motion, focus: 22 проверки.
- Матрица 1100×720 и 850×560 выполнена через viewport QA. Дополнительно
  подтверждены реальные нативные размеры main 1100×720 и resize до 850×560:
  `native-minimum-windows.json`; фактический Windows DPI обоих окон 120 (125%).
  DPR 100/125/150% проверен эмуляцией в обоих WebView (`dpi-emulation.log`),
  без overflow и clipped overlay buttons. Это не смена системного DPI.
- Скриншоты `output/stage-7/*-main-850.png`, `*-overlay.png`.
- Тестовые сигналы реакций (без media/Discord commands): next → listening,
  error → listening после таймеров, `mascot-timers.log`.
- Layout-only fixtures: 300 сессий, длинные title/artist/device, error/unavailable,
  disabled master; `release-stress.log`, `stress-*.png`. Это не доказательство
  наличия 300 реальных Windows сессий или реального сбоя WinRT.
- `21st review`: 0 errors, 0 warnings, 7 info suggestions (существующие цвета),
  `ui-review.json`. Общий дизайн не изменён. Keyboard focus видим, reduced motion
  поддерживается; маскот и обложки загружаются.

## Безопасность

- Добавлены только два read-only разрешения: `core:window:allow-is-visible` и
  `allow-is-minimized` для определения состояния собственного WebView.
- Overlay реально отклонил configure_startup, refresh_media_sessions,
  select_media_session, refresh_discord и open_sound_settings (ACL).
- Сохранены render-only аудиокоманды, source allowlist GSMTC, stale tokens/IDs,
  finite/range validation, bounded queues/deadlines, Try bool, integrity/path
  checks Discord, allowlist chords, cooldown, release injected keys и unknown.
- Новых интеграций, npm/Rust runtime dependencies, прав администратора,
  сетевой телеметрии, произвольных URI/путей или команд input не добавлено.
- Системные политики, UAC, Smart App Control и антивирус не изменялись.

## Ограничения и рекомендации перед этапом 8

- В текущем сеансе открыт голосовой раздел Discord. Реальные Mute/Deafen,
  закрытие Discord/Яндекс Музыки, изменение громкости и воспроизведения
  намеренно исключены из soak-сценария. Их живые управляющие проверки этапов
  4–6 не выдаются за повторные проверки этапа 7.
- Физический hotplug/исчезновение всех outputs, suspend/resume, смена default
  устройства во время звонка, multi-monitor/смена разрешения, elevated clients,
  PTB/Canary и exclusive fullscreen требуют отдельного безопасного сценария.
- Установка NSIS и настоящий вход в Windows с установленным автозапуском —
  ручные проверки. Рекомендуются перед распространением; защита не обходилась.
- Native thread names считаются каждые 30с. Точный COM reference count/event
  token heap profile не инструментирован: RAII/registration ownership проверены
  по коду; длительный тест не доказывает отсутствие любых возможных утечек.
- Низкоприоритетные риски из аудита не маскируются: повторное чтение thumbnail
  каждые 5с оставлено для recovery; поведение отдельных failed session rows
  требует hotplug/COM failure fixture до изменения рабочей семантики.
- Перед этапом 8 выполнить перечисленные hardware/installed-copy проверки,
  определить отдельное ТЗ. Этап 8 не запускался.

## Изменённые файлы

Production:

- `src/services/snapshotChannel.ts` — новый независимый координатор snapshots.
- `src/services/serviceSync.ts` — новый Tauri/Vue lifecycle adapter.
- `src/services/desktopSync.ts` — общий recovery и cleanup для desktop/theme.
- `src/stores/audio.ts`, `media.ts`, `discord.ts` — подключение синхронизации,
  безопасный error/recovery, подавление пропущенных временных реакций.
- `src-tauri/src/desktop.rs` — scoped события видимости show/hide/minimize.
- `src-tauri/capabilities/desktop-shared.json` — два read-only window permissions.
- `src/styles/mascot.css` — pause animations скрытого WebView.
- `src/styles/audio.css` — перенос длинных имён устройств.
- `vite.config.ts` — локальные ассеты вместо inline data-font.
- `src/components/layout/MainLayout.vue` — маркировка этапа 7.

QA/документация:

- `tests/snapshotChannel.test.ts` — 11 регрессионных тестов без новых libraries.
- `src-tauri/examples/hotkey_conflict.rs` — отдельный временный native fixture.
- `scripts/qa/stage7-metrics.ps1`, `stage7-threads.py` — дерево процессов,
  CPU/WS/Private/handles/threads и фактические имена трёх worker.
- `scripts/qa/stage7-soak.mjs`, `stage7-summary.mjs` — непрерывный тест/агрегация.
- `scripts/qa/stage7-layout.js`, `stage7-stress.js`, `stage7-dpi.js`,
  `stage7-listeners.js`, `stage7-hidden-regression.js` — воспроизводимые проверки.
- `README.md`, `context/CURRENT_STATE.md`, `docs/stage-7.md`.

Артефакты в `output/stage-7/`; исходные изображения не затрагивались.

Итоговые доступные измерения приведены ниже; полный 30-минутный прогон отменён пользователем.
Предварительный прогон `soak-process.jsonl` / `soak-ui.jsonl` прерван после
5,5 минут для исправления late-listen regression; `soak-final-*` — после 5 минут
для проверки target scope. Они не считаются 30-минутным тестом.

## Завершение доступных проверок · 16.10.2026

Пользователь отменил ожидание полного 30-минутного прогона. Новые функции не добавлялись: завершены холодные замеры, сравнение подписок и оформление отчёта. Production-код и установщик после прежних успешных сборок не менялись; повторная сборка того же кода не требовалась.

- Предыдущий непрерывный тест: 1396.405 с процесса / 1385.279 с UI (23 мин 16.4 с / 23 мин 5.3 с), 47 samples. Он был прерван по просьбе пользователя о паузе. `soak-30-summary.json` содержит 0 page errors/fatal, по одному worker, max visible revision lag=0. Hidden steady 455.095 с: root CPU 2.1493%, tree 3.7836% одного ядра, media mutations=0.
- В предыдущем тесте root private bytes 12,128,256 → 16,662,528 (max 16,941,056), handles 512 → 525 (max 591), threads 30 → 30 (max 51); tree private bytes 386,400,256 → 457,691,136 (max 484,335,616). Это наблюдаемый рост в сеансе с UI/DPR-проверками; он не доказывает утечку, но исключить её без более длительного профилирования нельзя. Отдельный новый холодный процесс показал 8.4–8.7 MiB private root; это другой процесс, а не «освобождение» памяти предыдущего.
- Новый короткий тест остановлен отдельно: UI 180.9 с / процесс 182.2 с, 7 samples. Не объединён с предыдущим. Яндекс Музыка оставалась paused, Discord mute/deafen=null; управляющие команды не отправлялись.
- До/после нового теста DOM listeners полностью совпали в обоих WebView: window popstate=1/pagehide=6/storage=1/focus=4; document mousedown=1/visibilitychange=4. Логи `listeners-resume-before.log`, `listeners-resume-after.log`. Это DOM listener counts, а не полный подсчёт внутренних WinRT/COM tokens.
- Временное QA-закрепление восстановлено: pin=false, autoHide=true; Neon Pink, Ctrl+D, пользовательский autostart и Discord-настройки сохранены (`resume-cleanup.log`).
- Диагностический процесс остановлен и выполнен обычный ручной запуск release без CDP. Эта остановка не выдаётся за штатный Exit: Exit через трей был отдельно подтверждён пользователем ранее.
- Финально PID 13860 (при следующем запуске определить заново), один lunea.exe, два Tauri Window, один tray_icon_app, по одному Audio/GSMTC/Discord worker. Main виден 1100×720, overlay скрыт. `final-native-windows.json`, `final-workers.json`. CDP 9223 не слушает; диагностических Vite/stage7-soak node-процессов не обнаружено.
- Установщик повторно проверен SHA256: совпадает с сохранённым артефактом. Новых runtime dependencies/permissions при завершении не добавлено; изменились только QA report helper и документация.

Полный 30-минутный тест не пройден и по последней просьбе не требуется для этого завершения. Доступная работа этапа 7 завершена; hardware/installed-copy/длительное heap-профилирование остаются явно указанными ограничениями. Этап 8 не запускался.
## Итоговые измерения

Baseline: исходный release этапа 6, оба окна скрыты, 60.98с, 3 samples.
Final: тот же сценарий после прогрева нового release, 122.56с, 5 samples.
Оба сравнения с локальным CDP; не строгий лабораторный benchmark, внешние приложения продолжали работать. В final Яндекс Музыка paused; снижение CPU дерева нельзя целиком приписать оптимизации.
CPU ниже — процент одного логического ядра; на данном ПК 12 logical processors.

| Метрика | Baseline | Final |
|---|---:|---:|
| CPU lunea.exe | 2.25% | 1.80% |
| CPU lunea.exe в нормировке Task Manager | 0.19% | 0.15% |
| Working Set lunea.exe, MiB | 18.6–19.3 | 17.6–42.4 |
| Private Bytes lunea.exe, MiB | 9.6–9.9 | 8.4–8.7 |
| Handles lunea.exe | 491–498 | 427–430 |
| Threads lunea.exe | 40–41 | 27–32 |
| CPU дерево LUNÉA + WebView2 | 11.58% | 2.52% |
| Сумма Private Bytes дерева, MiB | 289.3–326.4 | 261.5–265.3 |
| Processes в дереве | 8–8 | 8–8 |

Сумма Working Set процессов может повторно учитывать shared pages; Private Bytes и память самого lunea.exe приведены отдельно.
В первоначальном 120-секундном sampler CPU было ошибочно посчитано через обновляемый Process object; корректные delta восстановлены из cpuSeconds/time в baseline-corrected.json.
Файл baseline-hidden.jsonl с PID=0 исключён; используется только baseline-hidden-valid.jsonl с проверенным PID=20736.
Baseline command latency отдельно не сохранена, ускорение команд не заявляется.

## Непрерывный soak

Длительность UI-сценария: **180.9с**; серия процесса 182.2с, 7 samples, интервал около 30с.
Прогон остановлен по просьбе пользователя: полный 30-минутный тест отменён. Первые 3 минуты исключены из steady агрегатов как прогрев; поэтому в этом коротком прогоне steady hidden данных недостаточно. Отдельный прежний прогон 23 мин 16 с не объединяется с новым; его summary сохранён в soak-30-summary.json. В процессе выполнялись циклы main/overlay/both hidden и только cached state reads, без управления звуком/Discord/воспроизведением.

| lunea.exe | Начало → конец | Диапазон |
|---|---:|---:|
| Working Set, MiB | 18.2 → 20.3 | 18.2–20.3 |
| Private Bytes, MiB | 8.7 → 9.5 | 8.7–9.6 |
| Handles | 423 → 432 | 423–434 |
| Threads | 26 → 29 | 26–34 |

Во всех samples по одному lunea-core-audio / lunea-gsmtc / lunea-discord: **true**.
Page errors: 0; fatal сценария: 0; Discord оставался unknown: true.
Скрытые устойчивые интервалы: 0.0с; media mutations максимум нет достаточных данных за 30с.
Видимые media updates: median нет достаточных данных/с; maximum revision lag null.
Частота revisions зависит от пользовательского воспроизведения; paused не считается остановкой синхронизации.
Backend source IDs: ru.yandex.desktop.music.

| Cached state read | Median, ms | P95, ms | Max, ms |
|---|---:|---:|---:|
| audio | 15.8 | 47.5 | 47.5 |
| media | 8.0 | 16.4 | 16.4 |
| discord | 9.3 | 39.0 | 39.0 |

Это IPC + чтение cached DTO, а не latency реального volume/Toggle/WinRT управляющего действия. Точный COM heap/refcount profile не выполнялся; серия не доказывает отсутствие любых утечек.

## Release / NSIS

Финальная сборка: output/stage-7/release-scoped-build.log; тесты: rust-tests-final.log (13/13), frontend-tests-final.log (11/11), typecheck-final.log.
Установщик: src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe — **20,968,500 bytes**.
SHA-256: `3A95FE1786739FE98F6CF0CE2FCC99E6623974F31CEF8FD906B3FB863616CCFC`. Версия 0.1.0 сохранена, содержимое — этап 7. Установка автоматически не выполнялась.
Code Integrity 4551 в финальных проверках не повторилась; защита Windows не менялась.


