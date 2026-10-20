# LUNÉA — текущее состояние

## Этап 8 — фирменный установщик собран, 16.10.2026

Последнее указание пользователя: «Нет, только собрать установщик». НЕ устанавливать,
не обновлять и не удалять даже отдельную QA-копию без нового запроса. Подтверждение
не ожидается. Следующий этап не запускать автоматически.

- Реализован один custom NSIS Tauri пакет: native nsDialogs/MUI2, тёмный Neon Pink,
  оригинальный installer mascot, welcome/options/maintenance/instfiles/finish/error
  lifecycle, выбор папки/ярлыков/opt-in запуска. Основное Vue/Rust приложение и
  permissions не менялись. Версия 0.1.0, app.lunea.audio, x64/currentUser/HKCU сохранены.
- Предварительное удаление старой версии заменено на in-place maintenance в
  зарегистрированной папке. Staging .new.exe и backup .previous.exe, error guards,
  попытка rollback exe, запрет downgrade и незарегистрированной перезаписи.
  Это НЕ гарантия транзакции при power loss. Реальные update/uninstall не проверены.
- Оригинал C:/Users/riasg/Downloads/installer.png не изменён; копия 1312×1199 с alpha,
  SHA256 849814D2D5D87CEE1A55A242DF8FD9F67B9EE9B69833D3D3F1DF637D69838A80.
  Производные — пропорциональный resize + bitmap composition, без генерации/перекраски/
  обрезки. Восемь PNG приложения не менялись. Персонаж статичен, движений конечностей
  нет. Нет CSS/WebView анимаций; reduced motion соблюдается статичным интерфейсом.
- Код: src-tauri/tauri.conf.json; src-tauri/installer/{upstream.nsi,UPSTREAM.txt,
  LICENSE-MIT,template.nsi,lunea.nsh,assets/*}; scripts/installer/{prepare-assets.py,
  prepare-template.mjs,build-qa.ps1}; docs/stage-8.md, README.md, CURRENT_STATE.md.
- Проверки: npm run typecheck/build, cargo check, cargo test --lib 13/13,
  frontend regressions 11/11, полная release-компиляция и успешный NSIS bundle.
  Логи output/stage-8/. Native Welcome/Directory/Browse/Options/подтверждение отмены
  проверены, финальный Welcome screenshot welcome.png. DPI 125% фактический;
  физические 100/150% не проверены. Скриншотов Finish/progress/error нет: реальная
  установка отменена пользователем, не выдавать source-only страницы за пройденные.
- Реальный main/overlay WebView2 smoke прошёл: Audio ready, Media paused, Discord
  ready/unknown, Ctrl+D registered, no overflow/broken images. Темы не переписывались,
  полная theme matrix пройдена в этапе 7. Приложение возвращено к обычному запуску
  без CDP. Диагностические остановки не выдаются за штатный Exit.
- Пользовательские JSON до/после installer UI совпали по хешам: settings-preserved.json,
  equal=true. QA/production installer registration отсутствуют; пакеты не устанавливались.
  Autostart, звук, Discord и Яндекс Музыка не переключались.
- Подготовлена QA-копия с отдельными product/manufacturer/bundle IDs, но НЕ запускалась
  для установки. Автоматическая проверка сначала отклонила запуск («blocked by policy»),
  затем пользователь отказался от установочных тестов. Не пытаться обходить запрет.
- Подробная архитектура, полный file list, тесты, ограничения/инструкция:
  docs/stage-8.md. Обычный NSIS delete-app-data opt-in сохранён, но реальное удаление
  и сохранение данных при нём ещё не проверены. Перед распространением: VM matrix,
  новая/повторная/обновление/отмена/ошибка/удаление, WebView2 missing, DPI, подпись.

### Итоговый установщик этапа 8

src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe
21,447,874 bytes (20.45 MiB), версия 0.1.0, Authenticode NotSigned.
SHA256: 954A434E7420774F1D8EDE53254DEE424534D0BDF9135692231AE36732CCEB1F.
Метаданные output/stage-8/installer.json. Stage 7 setup сохранён в
output/stage-8/stage7-setup.exe. Подпись/Smart App Control/UAC/политики не обходились.
Для запуска вручную — двойной щелчок; сейчас пользователь запросил только сборку.

## Сохранённая история этапа 7 и предыдущих этапов
## Этап 7 — доступная работа завершена 16.10.2026

Пользователь возобновил этап 7, затем отменил ожидание 30-минутного прогона («мне кажется 30-минутный прогон уже не нужен»). Полный 30-минутный soak НЕ пройден. Доступные проверки и итоговые замеры выполнены, отчёт оформлен; этап 8 не запускать без нового задания.

### Выполнено

- Этапы 1–6 сохранены. SnapshotChannel/serviceSync обеспечивает watchdog 15 с, deadline 5 с, latest-only hidden views, revision guards, recovery ошибок и cleanup; resync не повторяет Toggle и старые реакции маскота.
- Scoped native visibility по label исправляет взаимное влияние main/overlay. Скрытые views не применяют snapshots и приостанавливают CSS-анимации; три Rust-сервиса продолжают работать.
- CSP data-font исправлена assetsInlineLimit=0 без ослабления CSP. Исправлен перенос длинного имени устройства. Темы, маскот и архитектура сохранены. Новых runtime dependencies нет; только два обоснованных read-only window permissions.
- Полный аудит, приоритеты, список файлов, безопасность, таблицы baseline/final, latency и ограничения: docs/stage-7.md. Машинные данные: output/stage-7/.

### Проверки и замеры

- Ранее пройдены npm run typecheck/build, cargo check, cargo test --lib 13/13, frontend regressions 11/11, npm run tauri build (release/NSIS). После них production-код не менялся, повторная сборка не требовалась.
- Реальный release WebView2: 22 UI-проверки, четыре темы, main 1100×720/850×560, overlay, focus/reduced motion; 0 page errors. 300 synthetic rows/long labels/error/unavailable проверены отдельно. Физический DPI 125%, 100/150% только эмуляция.
- 21st review: 0 errors, 0 warnings, 7 info. Проверены Ctrl+D, pin/autohide/minimize, autostart variants, single instance, конфликт hotkey, overlay ACL, stale token/session/volume validation. Пользовательский Exit: три worker stopped, процесс завершён; нефатальная WebView2 1412.
- Hidden regression: скрытые окна 0 snapshot updates при работающем backend; видимое продолжает обновляться. Late listen failure и revision/visibility races покрыты регрессиями.
- Холодный final: output/stage-7/cold-final.jsonl, 122.56 с, 5 samples. CPU root 1.80% одного ядра против baseline 2.25%; tree 2.52% против 11.58%. Root private 8.4–8.7 MiB против 9.6–9.9; handles 427–430 против 491–498; threads 27–32 против 40–41. Это короткое сравнение при работающих внешних приложениях, не строгий benchmark; на ПК 12 logical CPU. Final Яндекс Музыка paused, так что изменение tree CPU нельзя целиком приписывать оптимизации.
- Прежний прерванный soak: процесс 1396.405 с (23 мин 16.4 с), UI 1385.279 с, 47 samples, soak-30-summary.json. 0 page errors/fatal, один каждый worker, hidden media mutations=0; hidden CPU root 2.1493%, tree 3.7836% одного ядра.
- Прежний root private 12.1 → 16.7 MB, tree private 386.4 → 457.7 MB (десятичные MB). Наблюдаемый рост после UI QA не доказывает утечку, но отсутствие утечек не подтверждено; длительное heap/COM-профилирование остаётся ограничением.
- Новый soak-resume-* остановлен по последней просьбе: UI 180.9 с / процесс 182.2 с, 7 samples, 0 ошибок, worker один каждый. Прогоны НЕ объединены. После исключения первых 180 с steady hidden данных нового прогона недостаточно.
- DOM listeners до/после нового прогона полностью совпали: listeners-resume-before.log / listeners-resume-after.log. Это не полный COM/WinRT refcount profile.
- metrics-report.md сформирован stage7-metrics-report.mjs из нового summary; helper принимает путь summary первым аргументом и допускает отсутствие steady данных короткого теста.

### Финальное состояние среды

- Временные QA-настройки восстановлены: pin=false, autoHide=true, Neon Pink, Ctrl+D. Autostart уже был включён пользователем; регистрация не менялась. Discord keybinds/unknown сохранены. Звук, музыка и пользовательский звонок не переключались.
- QA Node/sampler остановлены. Release перезапущен обычным ручным запуском БЕЗ CDP. Последний PID 13860 — при продолжении обязательно определить заново, не доверять номеру.
- final-native-windows.json: один процесс, два Tauri Window, один tray_icon_app; main виден 1100×720, overlay скрыт. final-workers.json: Audio/GSMTC/Discord по одному. Порт 9223 не слушает; диагностических Vite/soak node процессов нет.
- Диагностическая остановка перед обычным запуском НЕ является проверкой штатного Exit. Штатный Exit через трей подтверждён ранее пользователем и debug-логом.

### Ограничения / дальнейшие действия

Новых обязательных задач этапа 7 по последней просьбе нет. Перед распространением рекомендуются безопасные ручные проверки: hardware hotplug/нет outputs/default switch/sleep-resume, несколько мониторов, физический DPI 100/150%, установка NSIS и новый вход Windows с автозапуском. PTB/Canary/elevated/fullscreen не проверены. Реальные изменяющие Audio/Media/Discord действия этапов 4–6 во время звонка на этапе 7 повторно не запускались. Полный длительный memory/COM profile не выполнен. Защита Windows и политики не менялись; 4551 в финальных проверках не повторилась.

### Установщик

src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe — код этапа 7 и этапов 1–6, версия 0.1.0, 20,968,500 bytes; сборка 16.10.2026 09:21:33 +04. SHA256 повторно совпал:
3A95FE1786739FE98F6CF0CE2FCC99E6623974F31CEF8FD906B3FB863616CCFC.
Установка автоматически не выполнялась. После сборки менялись только QA helper/документация.

### Изменённые и добавленные файлы этапа 7

Все пути ниже относительно C:/Users/riasg/Documents/ChatGPT/LUNÉA.

Production:
- src/services/snapshotChannel.ts — новый независимый координатор снимков.
- src/services/serviceSync.ts — новый Tauri-адаптер подписок и видимости.
- src/services/desktopSync.ts — общая синхронизация Desktop/тем.
- src/stores/audio.ts, src/stores/media.ts, src/stores/discord.ts — синхронизация и восстановление ошибок.
- src-tauri/src/desktop.rs — события нативной видимости.
- src-tauri/capabilities/desktop-shared.json — allow-is-visible и allow-is-minimized.
- src/styles/mascot.css — пауза анимаций скрытого окна.
- src/styles/audio.css — перенос длинного имени.
- vite.config.ts — assetsInlineLimit: 0.
- src/components/layout/MainLayout.vue — подпись этапа 7.

Проверки и инструменты:
- tests/snapshotChannel.test.ts — 11 регрессионных тестов.
- src-tauri/examples/hotkey_conflict.rs — временное резервирование сочетания для теста конфликта, без отправки ввода; не входит в установщик.
- scripts/qa/stage7-metrics.ps1 — CPU/RAM/handles/threads процесса и WebView, проверка PID/пути/start time.
- scripts/qa/stage7-threads.py — чтение имён Rust worker.
- scripts/qa/stage7-soak.mjs — циклы видимости и read-only snapshots.
- scripts/qa/stage7-summary.mjs — агрегация измерений.
- scripts/qa/stage7-metrics-report.mjs — итоговая таблица сформирована; cold-final.jsonl снят.
- scripts/qa/stage7-layout.js, stage7-stress.js, stage7-dpi.js, stage7-listeners.js, stage7-hidden-regression.js — UI, стресс, DPR, подписки, скрытые окна.

Документация: README.md, docs/stage-7.md, context/CURRENT_STATE.md. Логи и измерения: output/stage-7/. Git исходно содержал untracked файлы; не трактовать весь git status как изменения только этапа 7.


Дополнительно при завершении: scripts/qa/stage7-metrics-report.mjs — новый summary path и обработка отсутствующих метрик короткого теста; README.md/docs/stage-7.md/CURRENT_STATE обновлены.

## История этапов 4–6 (состояние до QA этапа 7)
- Этап 4: один Rust COM MTA worker; реальная default render громкость/mute; отдельные сессии всех активных выходов; input/output metadata. Backend events/revisions синхронизируют два WebView. Ввод/микрофон не изменяется.
- Этап 5: отдельный WinRT MTA MediaService, реальная desktop Яндекс Музыка через GSMTC. На ПК обнаружен точный source ID `ru.yandex.desktop.music` (Яндекс Музыка 5.122.0). Треки/обложки/Play/Pause/Next/Previous/Seek и timeline синхронизируют main/overlay через `lunea:media-state`. Браузеры не выбираются. В media.json сохраняется только source_id; выбранный режим после QA возвращён auto.
- Этап 6: отдельный DiscordService, стандартные verified process paths Stable/PTB/Canary, выбран по умолчанию Stable. На ПК Stable 1.0.9262, PTB/Canary отсутствуют. Официальные RPC/Social SDK voice settings требуют одобренной интеграции; LUNÉA использует fallback SendInput scan codes и пользовательские keybinds. Пользователь сам назначил и подтвердил Ctrl+Alt+Shift+F9/F10. discord.json сохраняет только настройки; сейчас enabled=true/confirmed=true. Mute/deafen всегда null (unknown), нет оптимистичных booleans/маскот mute по клику. Громкость конкретных Discord-сессий — прежний Core Audio. Демосостояние и update_demo удалены.
- Трей, выход и Ctrl+D этапа 3 пользователь подтвердил вручную. Автозапуск после входа отложен до установки. На этапе 4 Ctrl+D и реальный звук со скрытым main дополнительно проверены.
- Этап 5: typecheck/build, первые cargo test --lib 8/8, реальные debug WebView2 transport/metadata/cover/seek/4 темы/2 размера прошли. После защиты от старого timeline повторный тестовый exe заблокирован Code Integrity 4551; финальные debug и release/NSIS собраны. Release запущен: настоящий трек/обложка/GSMTC, два окна, одна иконка трея. Не отключать защиту и не изменять политики. Финальные результаты/артефакты — docs/stage-5.md.
- Реальные аудиотесты, файлы, API, ограничения и артефакты: [docs/stage-4.md](../docs/stage-4.md). QA probe — src-tauri/examples/audio_probe.rs, не часть приложения/установщика.
- Этап 6: финальные typecheck/build, cargo test --lib 13/13, debug WebView2, четыре темы и два main viewport прошли. Финальный release/NSIS собран, версия остаётся 0.1.0, содержимое этапа 6. F12 исключена как зарезервированная; whitelist Ctrl+Alt+Shift+F6–F11. SendInput проверен визуально в реальном Discord; двойной Toggle блокируется, отдельная сессия 100→73→100 не затрагивает другие приложения/master. После остановки Stable — unavailable, после запуска ready с новыми PID. Пользователь нажал «Выход» в трее: все три workers stopped, процесс=0; WebView2 дополнительно записал нефатальную Error=1412 при unregister Chrome_WidgetWin_0. На этапе 6 блокировка 4551 не повторилась.
- Остались физические hotplug/медиаклавиши этапа 4, установленный автозапуск после входа; multi-client/PTB/Canary/elevated/game environment этапа 6 не проверялись вживую. UI main viewports 1100×720 и 850×560, overlay с feedback до 360×440. Полного heap/COM-профилирования не было. Несколько реальных desktop-сессий GSMTC и одновременно активный foreign player на ПК не проверялись, политика выбора покрыта unit-тестами.
- Исходные аудиоуровни после QA восстановлены; регистрацию автозапуска не меняли; pin=false, autoHide=true, Neon Pink. Discord UI mic/deafen восстановлены в исходное выключенное состояние переключателей (обычные иконки); музыка продолжает воспроизведение. Подробности/ограничения/изменённые файлы: docs/stage-5.md и docs/stage-6.md. Не переходить к этапу 8 без задания пользователя.




