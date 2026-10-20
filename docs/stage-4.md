# Этап 4 — Windows Audio

## Результат и архитектура

Системный микшер заменён реальным Core Audio backend. Общая громкость и mute относятся к выходу Windows по умолчанию для роли eConsole. Сессии перечисляются на **всех активных устройствах вывода**, с названием устройства в каждой строке. Сессии одного процесса и одноимённые системные сессии разных выходов не объединяются: используется Windows session instance ID, отдельно передаётся deviceId.

Dashboard и overlay получают снимки одного Rust AudioService через `lunea:audio-state`. В каждом WebView собственная Pinia, но источником истины служит backend. Поля демогромкости удалены из desktop/demo-синхронизации и Rust DemoPatch: старые вызовы с volume/muted отклоняются. Музыка, смена трека, Discord Mute и Deafen остаются явно помеченными демонстрационными функциями.

Один поток `lunea-core-audio` инициализирует COM MTA. Только он создаёт, использует и освобождает COM-интерфейсы. Tauri commands передают ограниченные операции через bounded channel, ожидают результат вне главного потока и возвращают типизированные DTO/ошибки. Проверяются числовой диапазон, конечность f32, длина/содержимое ID, текущий выход и существование сессии. Команда со старым default device ID не изменит новое устройство. Сессионная команда проверяет доступность именно своего выхода.

Callbacks только устанавливают atomic dirty flag; не блокируют, не вызывают UI/COM и не сохраняют переданный Windows указатель сессии. Рабочий цикл объединяет уведомления (проверка до 40ms) и дополнительно сверяет состояние раз в 2 секунды: это нужно для гонок перечисления новых сессий, пропущенных уведомлений, ошибок регистрации и восстановления AudioSrv. Одна подписка устройств, одна endpoint-подписка для default output, один session notifier на каждый активный render endpoint и одна подписка на каждую наблюдаемую сессию. Повторное открытие окна их не создаёт. RAII снимает регистрации до CoUninitialize; RunEvent::Exit останавливает и join-ит рабочий поток.

Frontend объединяет быстрые движения слайдера, допускает одну запись в полёте на поле и сохраняет последнее оптимистичное положение, пока старые ответы ещё приходят. Канонические снимки имеют возрастающую revision; устаревшие ответы игнорируются. При исчезновении устройства/сессии её ожидающие UI-записи сбрасываются.

Состояния loading/ready/unavailable/error разделены. В обычном браузере показано, что Windows Audio доступен только в приложении, без фиктивных уровней. На Dashboard есть общий регулятор, отдельные mute/громкость сессий, прокручиваемый список, устройства вывода/ввода и их статусы/роли. Overlay содержит реальную общую громкость и адресный переход в полный микшер. Четыре прежние темы и MascotDisplay сохранены; сон реагирует на настоящий mute/ноль, listening — на активные немутированные сессии и прежнее демовоспроизведение. Active означает состояние потока Windows, а не анализ слышимости или содержимого звука.

Устройства показываются только для просмотра. Нет недокументированного изменения default endpoint: кнопка открывает фиксированный `ms-settings:sound`. Микрофоны не активируются для управления, их mute/громкость не изменяются. Ни звуковые данные, ни содержимое микрофона не захватываются, не записываются и не передаются.

## Windows API и команды

Использованы поддерживаемые `windows`/`windows-core` 0.62.2:

- IMMDeviceEnumerator: EnumAudioEndpoints, GetDefaultAudioEndpoint, GetDevice; IMMDevice: GetId/GetState/OpenPropertyStore/Activate; PKEY_Device_FriendlyName и PROPVARIANT с корректным освобождением.
- IAudioEndpointVolume: Get/SetMasterVolumeLevelScalar, Get/SetMute; IAudioEndpointVolumeCallback.
- IAudioSessionManager2, IAudioSessionEnumerator, IAudioSessionControl2: GetSessionInstanceIdentifier, GetState, GetProcessId, GetDisplayName, IsSystemSoundsSession; ISimpleAudioVolume для отдельной сессии.
- IMMNotificationClient, IAudioSessionNotification и IAudioSessionEvents, включая SimpleVolumeChanged/StateChanged/SessionDisconnected.
- CoInitializeEx(COINIT_MULTITHREADED), CoUninitialize, CoTaskMemFree; unregister/Drop всех наблюдателей на том же потоке.
- OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION), QueryFullProcessImageNameW для доступного имени процесса; ExtractIconExW/GetIconInfo/GetDIBits и DestroyIcon/DeleteObject/DeleteDC для маленьких локальных иконок. PNG/base64 нужны только для этих иконок, не для персонажа или аудиоданных. При недоступных метаданных используется имя Windows/PID и обычная иконка интерфейса.
- ShellExecuteW только с фиксированным URI настроек звука, без произвольного shell-доступа из JS.

Команды: get_audio_state, get_audio_sessions, get_audio_devices, refresh_audio, set_master_volume, set_master_mute, set_session_volume, set_session_mute; open_sound_settings только из main, open_mixer только из overlay. AppManifest и capabilities содержат узкие собственные permissions. Доступа к записи/захвату аудио, микрофону, произвольным файлам или процессам из frontend нет.

Документация: [Endpoint volume controls](https://learn.microsoft.com/en-us/windows/win32/coreaudio/endpoint-volume-controls), [Audio session events](https://learn.microsoft.com/en-us/windows/win32/coreaudio/audio-session-events), [IMMNotificationClient](https://learn.microsoft.com/en-us/windows/win32/api/mmdeviceapi/nn-mmdeviceapi-immnotificationclient).

## Изменённые файлы

| Файлы | Назначение |
|---|---|
| src-tauri/src/audio/mod.rs | DTO, AudioService, команды, bounded queue, запуск/остановка |
| src-tauri/src/audio/worker.rs | COM MTA, единый цикл, default output, все активные render endpoints, reconciliation |
| src-tauri/src/audio/devices.rs | Выходы/входы, роли, состояния, имена, освобождение строк/PROPVARIANT |
| src-tauri/src/audio/endpoint.rs | Общая громкость/mute и endpoint callback lifetime |
| src-tauri/src/audio/sessions.rs | Отдельные session instance IDs, события, метаданные и освобождение |
| src-tauri/src/audio/events.rs | Четыре вида Core Audio callbacks |
| src-tauri/src/audio/errors.rs | Типизированные ошибки и проверка входных значений, unit test |
| src-tauri/src/audio/icons.rs | Ограниченный доступ к метаданным процесса и маленьким иконкам |
| src-tauri/src/lib.rs | Регистрация аудио и shutdown при Exit |
| src-tauri/src/desktop.rs | Удаление фиктивной громкости/mute из Demo/Patch |
| src-tauri/build.rs | Перечень команд для ACL |
| src-tauri/permissions/desktop.toml | windows-audio, sound-settings, open_mixer |
| src-tauri/capabilities/desktop-shared.json, main-window.json | Разрешения по окнам |
| src-tauri/Cargo.toml, Cargo.lock | Windows bindings, windows-core для implement macro, png/base64 для иконок; npm-зависимости не добавлены |
| src-tauri/examples/audio_probe.rs | Отдельный явный QA-инструмент: независимое чтение, тестовые записи и две silent WASAPI-сессии; не запускается приложением и не включается в установщик |
| src/types/audio.ts, desktop.ts | Реальные аудиотипы, очистка demo DTO |
| src/stores/audio.ts | Канонические снимки, events, pending writes и защита от старых ответов |
| src/services/desktopSync.ts | Отделение аудио от demo; адресный переход к микшеру |
| src/components/ui/AudioMixer.vue | Реальный микшер и просмотр устройств |
| src/components/ui/OverlaySettings.vue | Актуальные подписи реального звука/демофункций |
| src/views/DashboardView.vue, OverlayView.vue | Подключение реальных контролов и перехода к микшеру |
| src/components/layout/MainLayout.vue | Реальный статус Windows Audio вместо общей demo-надписи |
| src/composables/useMascotReactions.ts | Реальный mute/active playback, сохранение приветствия |
| src/styles/audio.css, src/main.ts | Стили из существующих токенов, bootstrap AudioStore |
| README.md, docs/stage-4.md, context/CURRENT_STATE.md | Текущее состояние и отчёт |

## Проверки

Проверено в настоящем debug Tauri/WebView2 на Windows 11. Независимый Rust `audio_probe` читает Core Audio напрямую, минуя backend LUNÉA. Его fixture создаёт реальные WASAPI-сессии с тишиной, а не подменённые JSON-сессии. Основное приложение не использует внешние процессы для управления звуком.

| Проверка | Результат и доказательство |
|---|---|
| npm run typecheck | Пройден; output/stage-4-typecheck-delivery.log |
| npm run build | Пройден отдельно и внутри финального tauri build |
| cargo test --lib | 4/4, включая конечность/диапазон volume и ID; output/stage-4-tests-delivery.log |
| Debug Tauri | Реальный backend status=ready, выход Realtek, входы и их роли, процессы и настоящие иконки |
| Общая громкость из UI | 100→97%; независимый probe подтвердил 97, микрофон остался 1.0/unmuted |
| Системный mute | UI включил реальный mute; probe подтвердил true, уровень сохранён 97%; маскот перешёл в sleeping. Внешний unmute обработан |
| Внешнее изменение | Отдельный Core Audio probe установил 96%; main и overlay получили 96 без UI-refresh; затем восстановлено 100 |
| Браузер | Отдельная громкость 100→99 и mute; Яндекс Музыка осталась 80/unmuted, master=100. Браузер восстановлен в 100/unmuted |
| Две сессии одного PID | Реальные silent-сессии A/B: A=37/muted, B=100/unmuted, уникальные IDs и один PID; независимый probe подтвердил изоляцию. Повторено после финального расширения на все выходы |
| Все активные выходы | Обнаружены session device IDs двух настоящих выходов, включая отдельные системные сессии на наушниках и динамиках |
| Появление/закрытие приложения | Fixture добавил две строки; после естественного завершения обе исчезли без ручного обновления |
| Быстрый слайдер | 11 быстрых вводов, восемь последующих выборок стабильно 95%; overlay=95, затем восстановлено 100 |
| Скрытый main | Win32 visible=false; оверлей изменил реальную громкость до 97, скрытый main получил 97; probe подтвердил уровень и неизменный микрофон |
| Ctrl+D при скрытом main | Нативный Control_L+d скрыл overlay, оба HWND hidden, процесс жив. Повторный ручной запуск восстановил прежний main |
| Discord Mute/Deafen | Оба demo-переключателя включены при master=97; реальный endpoint не muted, микрофон остался 1.0/unmuted. Demo-переключатели восстановлены |
| Повторные открытия | После 12 toggle — один worker started в debug-журнале, один процесс, два Tauri Window и один tray_icon_app |
| Permissions/аргументы | 101% отклонено; чужой/default-stale ID и ID микрофона не приняты master-командой; закрытая сессия вернула session_closed; старый demo volume отклонён; overlay не может вызвать open_sound_settings |
| Четыре темы/адаптивность | Настоящий WebView2, эмуляция viewport 1100×720 и 850×560: без горизонтального overflow, картинки/иконки загружены. Native overlay 360×372 без overflow/cropping; page errors=[] |
| Полный микшер | Из overlay при main=/settings открылся обзор со скроллом к микшеру; overlay остался /overlay |
| 21st UI Review | 0 errors, 0 warnings, 7 info о прежних цветовых токенах; output/stage-4-review-final.json |
| Release / NSIS | Финальная сборка успешна, x64 19.86 MiB; output/stage-4-delivery.log. Code Integrity 4551 не повторилась |

Установщик: `src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe`. Установка и цифровая подпись не выполнялись. Системная защита и политики Windows не изменялись.

Все исходные уровни восстановлены: master=100/unmuted, Яндекс Музыка=80/unmuted, browser=100/unmuted; default microphone=1.0/unmuted и до, и после тестов. Pin=false, autoHide=true, Neon Pink; автозапуск не включался. Финальное независимое чтение: output/playwright/stage-4/final-windows-state.json.

Снимки, проверки и JSON находятся в исключённой из Git папке output/playwright/stage-4. UI snapshots main-*-850/1100.png и overlay-*.png; ui-results.txt, session-isolation.txt, isolation-final.txt, external-sync-results.txt, hidden-main-*.json/txt, observer-count.json, rapid-slider.txt и acl-and-input.txt. При принудительной перезагрузке WebView/dev-сервера возникали служебные IPC/unlisten warnings. Финальный штатный UI-проход не дал page errors. Устаревший модуль оверлея в долго работающем Vite был устранён перезапуском dev-сервера, затем тест синхронизации повторён успешно.

## Ограничения и ручная проверка

1. **Физические медиаклавиши:** помощник не поддержал XF86AudioLowerVolume/VolumeDown. Внешнее изменение Core Audio подтверждено, но нажатие аппаратной клавиши этим не считается. Вручную нажать клавиши громкости/mute при открытых main/overlay; сверить регуляторы и маскота, затем вернуть исходный уровень.
2. **Физическое подключение/отключение устройств не выполнялось.** Вручную отключить текущие наушники, проверить смену default output/недоступность, список устройств и исчезновение старых сессий. Вернуть наушники и проверить восстановление. Если невозможно отключить устройство, временно проверить другой доступный выход через стандартные параметры Windows; LUNÉA сама default device не меняет. Проверка отсутствия всех выходов также остаётся ручной.
3. **Завершение нового AudioService через меню трея** реализовано через RunEvent::Exit/join/RAII, но клик «Выход» в этом прогоне не автоматизирован. Ранее пользователь подтвердил ручную проверку трея/выхода этапа 3. Для этапа 4 повторить «Выход» при открытом микшере: процесс должен исчезнуть, в debug-журнале должно быть worker stopped; последующий запуск — без дублей. Принудительная остановка debug между сборками не считается проверкой этого сценария.
4. **Автозапуск после реального входа Windows** по прежнему решению пользователя отложен до установки. Проверок чистой Windows 10, эксклюзивного fullscreen и установки нового NSIS нет.
5. Защищённые/завершившиеся сессии могут не предоставлять имя/иконку/управление; недоступные данные пропускаются или имеют fallback. Idle-сессии доступны, пока Windows их перечисляет; expired удаляются. Новые очень короткие сессии, завершившиеся до следующего перечисления, не являются историей событий и не сохраняются.
6. У слайдера и mute нет управления микрофоном. Общая громкость относится к eConsole default output; индивидуальные сессии могут относиться к другим активным выходам, что видно в строке. Выбора нового системного default device внутри LUNÉA нет.

Skills: применены Frontend Design, UI/UX Pro Max (loading/error feedback, сохранение токенов и доступных подписей), 21st UI Review, Playwright CLI, Token Efficient Workflow и Context Compression. Точный «Ux UI Premium Direction» отсутствует; имеющийся Next.js Style Selector не применялся к существующему Vue-интерфейсу. Переустановки skills не было.

## Для этапа 5

Остаётся реальная медиаинтеграция по отдельному заданию: GSMTC/Яндекс Музыка, реальные метаданные трека и media controls. Discord RPC и фактические Mute/Deafen остаются до этапа 6. Текущий Core Audio уже регулирует доступные обычные сессии этих приложений. К следующему этапу автоматически не переходили.
