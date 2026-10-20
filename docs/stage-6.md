# Этап 6 — Discord

16.10.2026. Tauri 2 / Windows 11 / WebView2. Демонстрационные Discord booleans удалены. Используются пользовательские Toggle Mute/Deafen сочетания и существующий Core AudioService для громкости отдельных сессий. Этап 7 не начат.

Применены доступные Frontend Design, UI/UX Pro Max, 21st UI Review, Playwright CLI, Token Efficient Workflow и Context Compression (устойчивое состояние проекта в context/CURRENT_STATE.md). Точный skill «Ux UI Premium Direction» отсутствует; установленный ux-ui-premium-direction-style-selector предназначен для выбора нового направления Next.js и не применялся к сохранённой Vue-айдентике. Skills повторно не устанавливались.

## 1. Официальные механизмы и выбор

- Rich Presence предназначен для присутствия/активностей и сам по себе не предоставляет управление микрофоном.
- Документированный RPC содержит GET/SET_VOICE_SETTINGS и voice events. Это **не означает свободного доступа**: RPC для непринятых приложений ограничен зарегистрированными тестерами, а voice read/write требуют одобряемых разрешений и авторизации приложения. У LUNÉA нет зарегистрированной/одобренной интеграции. [Официальная документация RPC](https://github.com/discord/discord-api-docs/blob/main/developers/topics/rpc.mdx), [OAuth scopes](https://discord.com/developers/docs/topics/oauth2).
- Game SDK — прежний механизм; для новых проектов Discord рекомендует Social SDK. [Официальный Game SDK](https://github.com/discord/discord-api-docs/blob/main/developers/developer-tools/game-sdk.mdx).
- Актуальный Social SDK 1.10.19337 от 01.09.2026 добавил read-only доступ к настройкам desktop voice. `Client::GetVoiceSettings` требует работающий клиент и одобренную Social SDK integration. Callback позволяет получать изменения, но без такой регистрации здесь нельзя объявлять состояние доступным. SDK self-mute/deaf методы для Calls/Lobbies не следует выдавать за универсальное управление произвольным desktop-звонком. [Release notes](https://discord.com/developers/docs/social-sdk/release_notes.html), [Client reference](https://discord.com/developers/docs/social-sdk/classdiscordpp_1_1Client.html), [Voice guide](https://github.com/discord/discord-api-docs/blob/main/developers/discord-social-sdk/development-guides/managing-voice-chat.mdx).
- Выбран предусмотренный ТЗ fallback: штатные пользовательские глобальные keybinds desktop Discord, отправляемые документированным Windows SendInput. Клавиши пользователь назначает самостоятельно; LUNÉA не редактирует настройки Discord. [Discord keybinds](https://discord.com/blog/how-to-use-keyboard-shortcuts-on-discord-create-custom-keybinds), [Microsoft SendInput/UIPI](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput).

Не использованы user token, self-bot, приватные API, перехват трафика, чтение памяти, модификация Discord или автоматизация браузерной версии. SDK/RPC не подключены тайно и не эмулируются.

## 2. Обнаружение и архитектура

Новый singleton DiscordService имеет собственный небольшой worker и `lunea:discord-state` с возрастающим revision. Он не создаёт Core Audio или GSMTC worker. Snapshot содержит проверенные процессы, настройки, выбранные аудиосессии и последнюю попытку отправки. Два WebView получают событие и начальное состояние независимо.

Для Stable/PTB/Canary проверяются стандартные каталоги LOCALAPPDATA / Program Files / Program Files (x86), структура `Discord[PTB|Canary]/app-*/Discord[PTB|Canary].exe`. Running подтверждается ToolHelp process enumeration и QueryFullProcessImageNameW, а не заголовком окна. Helper и другие exe не считаются клиентом. Установка без процесса не считается подключением. Вариант по умолчанию — Stable; PTB/Canary выбираются явно, без автоматической подмены.

На ПК обнаружен **Discord Stable 1.0.9262**, исходный путь `C:\Users\riasg\AppData\Local\Discord\app-1.0.9262\Discord.exe`. PTB/Canary не установлены. Несколько renderer-процессов одного клиента объединяются только в набор проверенных PID для обнаружения; их аудиосессии остаются раздельными.

Если работают несколько вариантов Discord, отправка блокируется даже после выбора одного: SendInput глобален и не может гарантировать адресацию только выбранному клиенту. Пользователь должен закрыть остальные варианты. Кастомные/portable каталоги намеренно не принимаются как проверенные; произвольный путь из frontend не используется.

Core Audio DTO читаются из существующего AudioService. Discord-сессии сопоставляются по проверенному PID выбранного клиента. Dashboard управляет конкретной строкой через прежние set_session_volume/set_session_mute. Устройства/сессии не меняются автоматически, действия «изменить всё» нет. Mute сессии Windows явно отделён от Deafen.

## 3. Ввод, безопасность и неизвестное состояние

Пустая конфигурация по умолчанию отключена. Пользователь подтверждает самостоятельное назначение клавиш в выбранном Discord. Хранятся только client, enabled, confirmed, mute, deafen в `%APPDATA%/app.lunea.audio/discord.json`. Секретов нет.

Whitelist: Ctrl + Alt + Shift + одна F6–F11, порядок модификаторов нормализуется. Ограничена длина, запрещены повторные модификаторы, текст/скрипты, Win, одиночные клавиши, другие chords и одинаковые Mute/Deafen. Тем самым Win+L, Ctrl+Alt+Del, Alt+F4 и системные сочетания не принимаются. Другие программы могут иметь пользовательское назначение на разрешённый chord — LUNÉA не обещает обнаружить все чужие keybinds и предупреждает об этом.

Конфликт с горячей клавишей оверлея проверяется при сохранении Discord, при изменении overlay shortcut и непосредственно перед отправкой. Discord-chords LUNÉA не регистрирует глобально и не перехватывает: их слушает сам Discord. Одновременные изменения настроек также защищены повторной проверкой в момент отправки.

F12 дополнительно исключена из allowlist: Microsoft резервирует её для отладчика. Ваши F9/F10 это уточнение не меняет. [Microsoft RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey).

Frontend передаёт только фиксированное действие Mute/Deafen, ожидаемый клиент и configuration revision. Саму последовательность input events frontend задавать не может. Перед SendInput вновь проверяются running/path/client/settings; старые конфигурации отклоняются. Общий backend cooldown 1,2 с защищает между двумя окнами; frontend подавляет частые клики. Очередь/deadline ограничены, при старте input не отправляется.

Ввод использует scan codes через MapVirtualKeyW, KEYEVENTF_SCANCODE и KEYEVENTF_KEYUP. Модификаторы отпускаются в обратном порядке. При частичной вставке рассчитывается только реально введённый prefix; выполняется bounded cleanup отпускания собственных нажатий, без отпускания заранее удерживаемых пользователем клавиш. Если модификатор/назначенная F-клавиша уже удерживается, команда не отправляется.

GetTokenInformation(TokenIntegrityLevel) проверяет уровень Discord и активного окна относительно LUNÉA. Если права выше или сведения недоступны, отправка блокируется. Администратор не требуется и автоматически не запрашивается; UIPI и античит не обходятся. Windows может блокировать input/cleanup даже при корректном коде; это нельзя превращать в гарантию переключения.

`mute` и `deafen` в текущем режиме всегда **null**. Успешная вставка всех events означает только `lastAttempt.status=sent` и «Сочетание отправлено в Windows. Переключение Discord не подтверждено». Отказ — `not_sent` с причиной. В UI нет optimistic booleans и aria-pressed для Discord toggle-действий. Подтверждённое состояние этим режимом не предоставляется.

Маскот не переходит в mute по нажатию; отправка с ошибкой вызывает временный error, затем базовую позу. Системный mute/ноль сохраняет приоритет sleeping. Существующие PNG, темы, MediaService и Core Audio модули сохранены.

## 4. Интерфейс и команды

Settings: выбранный Stable/PTB/Canary, обнаружение, режим, поля Mute/Deafen, inline validation, подтверждение, включение, последняя попытка и инструкция. На установленном Discord новый путь — «Настройки пользователя → Система → Пользовательские горячие клавиши». На открытой странице настройки keybinds Discord отключает их выполнение; это указано в инструкции. Кнопка из карточки ведёт к разделу Discord в Settings.

Dashboard: действия, неизвестное состояние, результат, отдельные громкости/mute и устройства каждой сессии. Overlay: нейтральные действия «Микрофон»/«Звук», unknown и сообщение о доставке; существующая адаптация высоты, максимум 360×440.

Tauri commands: get_discord_state, get_discord_settings, configure_discord, refresh_discord, discord_toggle_mute, discord_toggle_deafen. `discord-controls` даёт двум окнам чтение и два фиксированных действия; `discord-settings` даёт только main configure/refresh. Raw input, shell, filesystem, admin или remote permissions не добавлены. Включены только нужные features существующего windows crate; новых npm/Rust-библиотек нет.

## 5. Проверки

| Проверка | Результат |
| --- | --- |
| npm run typecheck / npm run build | Успешно, включая новую Vue/Pinia интеграцию |
| cargo check | Успешно |
| cargo test --lib | Финальный проход **13/13**, включая allowlist/chord conflicts, partial-input cleanup prefixes, проверку process path и выбор сессий по PID |
| Debug Tauri/WebView2 | Настоящие Core Audio, GSMTC и Discord DTO в двух окнах |
| Stable/PTB/Canary | Реальный Stable обнаружен по exe; PTB/Canary отсутствуют, явный выбор PTB даёт unavailable. Работа установленных PTB/Canary не проверялась |
| Клавиши | Пользователь сам назначил и подтвердил Ctrl+Alt+Shift+F9/F10. QA не редактировал назначения Discord |
| Реальный Mute/Deafen | SendInput со scan codes; визуально проверены красный microphone и обе красные mic/headphone иконки. Проверены вызовы при Discord и через интерфейс LUNÉA с другим активным окном. Доставка/визуальное наблюдение QA не превращены в постоянное состояние продукта |
| Двойной Toggle | Два одновременных backend запроса: один fulfilled, второй rejected cooldown. Повтор не отправлен; error синхронизирован в двух окнах |
| Audio isolation | Две реальные сессии Discord на разных выходах. Одной задано 73% и mute; master и все прочие сессии неизменны, затем уровень/mute восстановлены. После перезапуска Discord сессии повторно обнаружены с новыми PID |
| Конфликты/невалидный ввод | Win+L, Ctrl+Alt+Del, текст, duplicate, неподтверждённая конфигурация, invalid variant и stale revision отклоняются. Попытка назначить overlay chord Discord отклонена до регистрации |
| ACL | configure_discord / refresh_discord из overlay отклонены permissions Tauri |
| Исчезновение | Завершены только проверенные Stable-процессы, не произвольный Discord helper. Main/overlay: installed=true, running=false, unavailable, canSend=false, audioSessions=[], кнопки disabled |
| Восстановление | После запуска того же exe новые PID автоматически обнаружены, ready без перезапуска LUNÉA; сохранённые keybinds не менялись |
| Main скрыт | Close main → скрыто, overlay Deafen-команда отправлена; состояние осталось unknown. Повторная команда использована для восстановления; основное окно открыто обратно |
| Синхронизация | Одинаковые lastAttempt/preferences/null mute/deafen и revision 35 после восстановления, два отдельных WebView |
| Музыка/Core Audio | GSMTC продолжал playing с реальными track/cover; master 100%, muted=false; Яндекс-сессия 80%, Discord 100% после QA |
| Четыре темы/размеры | WebView main 1100×720 и 850×560, overlay до 360×440 с feedback. Нет horizontal overflow/clipping controls, page errors=0; нейтральные кнопки без aria-pressed |
| 21st review | 0 errors, 0 warnings; 7 info suggestions о существующих цветах main.css |
| Выход | Пользователь выбрал «Выход» в трее. Лог: Discord worker stopped → GSMTC worker stopped → Windows Audio worker stopped; lunea process count=0. Это штатный Exit, не принудительный stop |

Артефакты: `output/stage-6-{detection.json,check.log,typecheck-final.log,build.log,tests-complete.log,review.json}`, `output/playwright/stage-6/{configured,behavior,rejections,ui,discord-closed,recovered}.log`, скриншоты UI рядом, `output/stage-6-final-runtime.log` содержит подтверждённый shutdown. QA example `src-tauri/examples/discord_input_probe.rs` не запускается приложением и не входит в установщик; без аргумента только discovery, с `mute`/`deafen` — только два согласованных тестовых chord.

## 6. Изменённые файлы

Новые:

- `src-tauri/src/discord/{mod,detection,hotkeys,input,sessions,errors}.rs`.
- `src-tauri/examples/discord_input_probe.rs`.
- `src/types/discord.ts`, `src/stores/discord.ts`, `src/styles/discord.css`.
- `src/components/ui/{DiscordActions,DiscordCard,DiscordSettings}.vue`.
- `docs/stage-6.md`.

Изменены:

- `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/src/{lib,desktop}.rs`.
- `src-tauri/permissions/desktop.toml`, `src-tauri/capabilities/{main-window,desktop-shared}.json`.
- `src/main.ts`, `src/types/desktop.ts`, `src/services/desktopSync.ts`.
- `src/views/{DashboardView,SettingsView,OverlayView}.vue`.
- `src/components/ui/OverlaySettings.vue`, `src/components/layout/MainLayout.vue`, `src/composables/useMascotReactions.ts`.
- `README.md`, `context/CURRENT_STATE.md`.

Удалён `src/stores/demo.ts`; backend Demo/DemoPatch/update_demo и их permissions тоже удалены. Cargo.lock не требует новых зависимостей: использованы features существующего windows crate. Исходные аудио/медиа-модули и PNG не изменены.

## 7. Ограничения и ошибки

- Unknown сохраняется даже после успешного теста: внешние нажатия/изменения Discord, временная блокировка hotkeys и UIPI исключают достоверный локальный boolean.
- Передавать input конкретному выбранному из одновременно работающих клиентов нельзя; поэтому multi-variant send запрещён. Реальных PTB/Canary и одновременно нескольких клиентов на ПК нет, негативные правила покрыты unit/code review.
- Глобальные chord могут использоваться другой программой. Allowlist исключает штатные опасные системные комбинации, но пользователь должен выбирать свободные назначения. Игра/античит, elevated foreground, secure desktop и сторонние hooks могут блокировать доставку. Проверка не обходила эти ограничения и не повышала права.
- Принудительные частичные SendInput failures и elevated Discord специально не создавались. Cleanup проверен по каждому prefix unit-тестом, реальная UIPI-несовместимость не воспроизводилась.
- Discovery поддерживает проверенные стандартные Windows installation roots; portable/custom installation может показываться как не обнаруженная.
- Реальное переключение проверялось на обычном desktop Discord вне активного голосового звонка. Во время живого звонка/эксклюзивного fullscreen/игры отдельно не проверено. В голосовой канал QA не входил и сообщений не отправлял.
- При штатном Exit WebView2 записал `Failed to unregister class Chrome_WidgetWin_0, Error=1412`. Все три worker остановлены, процесс завершился. Это отдельная нефатальная диагностика WebView2, а не зависший DiscordService.
- Установщик/повторный вход в Windows/автозапуск потребуют пользовательской проверки установленной копии. Автозапуск и существующие desktop preferences не менялись.

## 8. Release и дальнейшая работа

Release/NSIS после интеграции собраны успешно, затем пересобраны после исключения F12. Финальные логи: `output/stage-6-release-verified.log`, `output/stage-6-tests-verified.log` (13/13), `output/stage-6-typecheck-verified.log`. Frontend build входит в эту Tauri-сборку. Установщик: `src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe`; версия 0.1.0 сохранена, содержимое — этап 6.

Smoke-check финального release: native accessibility показала «ЭТАП 6», реальную музыку GSMTC, «Discord запущен», доступные Toggle-действия и «Состояние неизвестно». После штатного перезапуска сохранённые F9/F10/Stable/enabled/confirmed восстановлены без автоматической отправки input. Два Tauri окна, одна tray_icon_app, один процесс; `output/playwright/stage-6/release-final-windows.json`. Итоговый release запущен для передачи пользователю. Vite/debug остановлены. Установщик не устанавливался автоматически. Финальный NSIS — 20 968 670 байт (~20 МиБ).

Code Integrity 4551 на этапе 6 не повторилась. Защита Windows и системные политики не менялись. Автозапуск и прочие desktop preferences сохранены; пользовательские F9/F10 остаются включены в LUNÉA после явного подтверждения пользователем.

ТЗ этапа 7 пока нет. Возможное отдельное направление — зарегистрированная и одобренная Social SDK интеграция для подтверждённого read-only voice state; это потребует официальной регистрации/разрешений и отдельного решения пользователя. Текущий fallback не выдаёт это за уже реализованную функцию. Этап 7 автоматически не запускается.
