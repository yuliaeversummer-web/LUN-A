# Этап 5 — реальная Яндекс Музыка через Windows GSMTC

Проверено 16.10.2026 на текущем Windows 11 / WebView2. Реальные музыкальные controls заменили демонстрацию; Discord Mute/Deafen остаются демо. Этап 6 не начат.

## Источник и API

Read-only probe `src-tauri/examples/media_probe.rs` обнаружил **`ru.yandex.desktop.music`** у установленного приложения `C:\Users\riasg\AppData\Local\Programs\YandexMusic\Яндекс Музыка.exe`. Версия в его интерфейсе: 5.122.0. Начальное чтение: Savage — Don't Cry Tonight, playing, PNG thumbnail 53 100 байт. ID взят из реальной сессии, а не угадан по названию трека.

Использованы bindings windows 0.62.2: `GlobalSystemMediaTransportControlsSessionManager.RequestAsync`, `GetSessions`, `GetCurrentSession`; `SourceAppUserModelId`, `TryGetMediaPropertiesAsync`, `GetPlaybackInfo`, `GetTimelineProperties`; `TryPlayAsync`, `TryPauseAsync`, `TryTogglePlayPauseAsync`, `TrySkipNextAsync`, `TrySkipPreviousAsync`, `TryChangePlaybackPositionAsync`. Подписки: SessionsChanged, CurrentSessionChanged, MediaPropertiesChanged, PlaybackInfoChanged, TimelinePropertiesChanged; каждый event token снимается при Drop.

В этой версии API нет `GetSessionById`; собственный поиск выполняется по непрозрачному runtime ID из реестра обнаруженных сессий. Свойство источника называется `SourceAppUserModelId`, а не `GetSourceAppUserModelId`. Список методов подтверждён [документацией Microsoft](https://learn.microsoft.com/en-us/uwp/api/windows.media.control.globalsystemmediatransportcontrolssessionmanager?view=winrt-26100) и установленными Rust bindings.

Только точное совпадение с обнаруженным desktop ID допускается к metadata/controls. Автовыбор: единственная подходящая сессия; при нескольких — текущая Windows, только если она сама подходит; иначе неоднозначность и ручной выбор. Браузерная current session не становится fallback. Ручной выбор удерживает живую выбранную сессию; после её исчезновения разрешено восстановление только по единственному совпадению устойчивого source ID. При нескольких совпадениях нужен повторный выбор. В `%APPDATA%/app.lunea.audio/media.json` сохраняется только `source_id`; COM-указатели и временные IDs не сохраняются.

## Архитектура и поведение

Отдельный singleton MediaService работает в одном WinRT MTA-потоке. Core AudioService не менялся. GSMTC callbacks выставляют dirty flags; сверка discovery — каждые 2 секунды, metadata дополнительно каждые 5 секунд. Единственный backend-цикл обновляет timeline каждые 250 мс. Событие `lunea:media-state` имеет возрастающий revision; каждый WebView имеет собственный MediaStore и начальный запрос. Frontend не заводит часы прогресса.

WinRT async ожидается вне UI-потока с проверкой IAsyncInfo и Cancel при остановке/таймауте 3 секунды. Очередь ограничена 32 запросами, команды имеют deadline. После трёх последовательных ошибок чтения manager пересоздаётся, старые подписки освобождаются. При штатном Exit сервис получает stop, снимает подписки, освобождает WinRT и присоединяет поток.

Перед transport-командой выполняются свежая discovery, проверка ожидаемого opaque ID и exact source ID, затем свежие Controls. Учитывается boolean результата Try...Async. Seek проверяет конечное число, доступный диапазон, track token и свежие title/artist/album, переводит относительные миллисекунды в 100-нс ticks с checked arithmetic. Состояние playing, позиция и трек не назначаются оптимистично.

Позиция вычисляется из LastUpdatedTime/PlaybackRate, ограничена длительностью; пауза её не увеличивает. Оценка помечена `≈`. При ошибке показывается подтверждённая позиция. При смене title прежняя позиция скрывается до изменения timestamp Windows; дополнительно отбрасывается прежняя длительность с позицией около предыдущей. Одного нового timestamp оказалось недостаточно: плеер иногда сначала обновляет timestamp старого timeline. Ожидание также отключает seek. Если две композиции имеют одинаковую длительность и переключены в самом начале, эта консервативная проверка может оставить прогресс недоступным до следующего различимого обновления источника.

Обложка читается только из WinRT thumbnail/stream: PNG/JPEG/WebP, до 512 КиБ, размеры до 2048×2048. Потоки/reader закрываются. В памяти хранится одна текущая обложка; отдельный `get_media_artwork` отдаёт её только для текущего artwork ID. Base64 thumbnail не входит в частые timeline-события, на диск не пишется. Frontend очищает картинку до нового чтения и отбрасывает запоздалые ответы. При отсутствии обложки используется существующий знак LUNÉA.

Dashboard и overlay используют общий MediaCard; Settings содержит обнаружение и ручной выбор. Сохранены четыре темы и существующий MascotDisplay. Play → listening, Pause → idle; принятые Next/Previous → next на 1,8 с; ошибка команды → error на 3,5 с. Переход Loading → Playing не отменяет временную реакцию. Системный mute/ноль имеет безусловный приоритет sleeping. Unavailable не вызывает постоянную ошибку маскота.

Permissions: `media-controls` для двух окон — только чтение и transport; `media-settings` только main — refresh/select. Нет новых filesystem/shell/remote permissions. Дополнительная прямая зависимость windows-future 0.3.2 уже была транзитивной; добавлены необходимые features существующего windows crate. Npm-зависимости не добавлялись.

## Изменённые файлы

Новые:

- `src-tauri/src/media/{mod,manager,sessions,metadata,artwork,events,errors,policy}.rs`.
- `src-tauri/examples/media_probe.rs` — read-only QA, не запускается приложением и не входит в установщик.
- `src/types/media.ts`, `src/stores/media.ts`.
- `src/components/ui/MediaCard.vue`, `src/components/ui/MediaSettings.vue`, `src/styles/media.css`.
- `docs/stage-5.md`.

Изменены:

- `src-tauri/{Cargo.toml,Cargo.lock,build.rs}`, `src-tauri/src/{lib,desktop}.rs`.
- `src-tauri/permissions/desktop.toml`, `src-tauri/capabilities/{desktop-shared,main-window}.json`.
- `src/main.ts`, `src/types/desktop.ts`, `src/stores/demo.ts`, `src/services/desktopSync.ts` — музыка удалена из демосостояния, Discord сохранён.
- `src/views/{DashboardView,OverlayView,SettingsView}.vue`.
- `src/components/layout/MainLayout.vue`, `src/components/ui/OverlaySettings.vue`, `src/composables/useMascotReactions.ts`.
- `README.md`, `context/CURRENT_STATE.md`.

Оригинальные PNG маскота, его компонент, Core Audio модули и система тем не переписывались. QA-скрипты/скриншоты/логи находятся в игнорируемом `output/`.

## Реальные проверки

| Проверка | Результат |
| --- | --- |
| npm typecheck / production frontend build | Успешно; повторный frontend build входит в финальную Tauri-сборку |
| cargo test --lib | Первые проходы: 8/8, включая выбор источника, аргументы seek и интерполяцию. Последний повтор после защиты timeline скомпилировался, но запуск тестового exe заблокирован Windows: os error 4551. Это не успешный финальный test run |
| Debug Tauri/WebView2 | Реальные metadata, cover 150×150, transport, Settings, два окна проверены через Playwright CDP |
| Play/Pause | UI-команда Pause подтверждена независимым read-only probe: PlaybackStatus=5; Play — PlaybackStatus=4. На паузе позиция 33 333 мс неизменна через 1,2 с |
| Next/Previous | Через LUNÉA: «Вика» → «немерено» → «Вика», новые track tokens/обложки. Try...Async принято |
| Seek | Через range UI на 30 000 мс, Windows затем вернул подтверждённую позицию 30 281 мс |
| Внешняя смена трека | Нативная кнопка «Следующая песня» в самой Яндекс Музыке: «Мой ангел» → «Температура»; main/overlay обновились без перезапуска |
| Закрытие/восстановление источника | Для удаления фоновой GSMTC-сессии завершены процессы только по проверенному пути exe, затем приложение запущено вновь. Оба окна: unavailable, track/timeline=null, covers=0. После Play в самом плеере автоматически ready с новым runtime ID |
| Ручной выбор | Радиокнопка Settings, JSON только source_id. После перезапуска LUNÉA восстановлен manual при новом runtime ID; после QA возвращён auto |
| Отказы | Устаревший token → track_changed; вне диапазона → invalid_position; чужой ID → session_changed; Play в playing при play=false → unsupported; старая обложка → stale_artwork. Управления другим плеером не было |
| ACL overlay | refresh_media_sessions отклонён Tauri как main-only |
| Main/overlay | Одинаковые revision/track/caps/позиция и загруженная обложка; две независимые JS-среды |
| Скрытый main | Нативная диагностика: main invisible; Ctrl+D из окна Яндекс Музыки показал overlay. Transport overlay работает при скрытом main |
| Повторные открытия | 12 hide/show циклов, один backend worker start и одна пара Tauri окон/иконка трея. Полного heap/COM-профилирования не проводилось |
| Core Audio | Master 100→99→100; mute восстановлен. Уровни/mute всех аудиосессий до/после одинаковы. Sleeping в обоих окнах даже во время media error |
| Маскот | Next остаётся после обновления ready и возвращается в listening после таймера; системный mute приоритетен |
| Темы/размеры | Все 4 темы, main WebView viewports 1100×720 и 850×560, native overlay 360×400. Нет горизонтального overflow, обрезания controls и незагруженных covers; page errors=0 |
| 21st UI Review | 0 errors, 0 warnings, 7 informational suggestions о старых цветах в main.css |

Артефакты: `output/stage-5-discovery.json`, `output/stage-5-tests.log`, `output/stage-5-tests-final.log`, `output/playwright/stage-5/{transport,pause-stability,ui,audio-windows,yandex-stopped,yandex-recovered,external-next,final-mascot}.log`, screenshots рядом. Последние дополнительные проверки timeline/release описаны в финальном дополнении ниже.

## Ограничения и оставшиеся проверки

- Установленная Яндекс Музыка не заполнила AlbumTitle у проверенных треков; поле скрыто. Caps динамические: Play=false при playing, Pause=false при paused, Previous бывает false на первом треке волны. Это ограничения источника, UI их учитывает.
- Холодный запуск плеера может не публиковать GSMTC до первого Play. Закрытие окна обычно оставляет фоновый процесс/сессию; тест удаления сессии выше — именно завершение процессов, а не доказательство поведения кнопки X.
- Сам плеер показал предупреждение о нестабильной работе сервиса. Метаданные/обложка/timeline могут приходить раздельно. Thumbnail отсутствует или ещё не готов — placeholder; ожидание нового timeline — `--:--`, без старой перемотки.
- На ПК была одна подходящая GSMTC-сессия. Неоднозначность нескольких desktop-сессий и foreign-current проверены unit-тестами; одновременная реальная сессия браузера/Spotify не создавалась. Иные варианты desktop AppUserModelId намеренно не угадываются.
- Повтор «Выход» через нативное меню трея с новым MediaService остаётся ручной проверкой. Код shutdown привязан к RunEvent::Exit и снимает подписки, но принудительная остановка процесса при пересборке не считается этим тестом. Аналогично не заявляется полноценное доказательство отсутствия любых утечек по одному worker count.
- Повреждение WinRT manager, отказ Try=false, недоступность отдельных событий, oversized/invalid thumbnails обработаны кодом; искусственно в реальной Яндекс Музыке не воспроизводились.
- Установка NSIS, повторный вход в Windows и автозапуск остаются ручными. Системная защита не менялась. Нельзя обещать overlay поверх exclusive fullscreen.
- Точный текущий трек/очередь до QA после завершения Яндекс Музыки восстановить нельзя через GSMTC: библиотека и произвольный выбор трека не входят в этот API. Громкость и настройки LUNÉA восстановлены; музыка оставлена в воспроизведении.

Этап 6: отдельная реальная интеграция Discord и замена оставшихся demo Mute/Deafen после нового задания пользователя. Библиотека Яндекса, авторизация, скачивание, DRM и захват аудио не добавлялись.

## Финальное дополнение проверки

Финальный debug после защиты timeline собран и запущен. Реальная смена «Пить-плакать» → «Буду Вуду»: при новом названии timeline=null, затем источник пересоздал сессию и дал новую длительность 253 770 мс с позицией 420 мс. Старые 130 секунд от предыдущего трека не показывались под новым title. Next сохранялся до завершения временной реакции, затем listening. Повторный ручной выбор той же текущей сессии сохранил доступный timeline; после проверки возвращён auto. Артефакт `output/playwright/stage-5/timeline-final.log`.

Ошибка целевой команды: error → listening после таймера; визуальный stress-test длинных названий в обоих окнах: ellipsis, без document overflow. Это временная подмена DOM только для layout QA, не демоданные продукта. Громкость в конце 100%, muted=false, Neon Pink, pinned=false, autoHide=true. Существующая регистрация автозапуска оказалась включена, startInTray=true; обе настройки сохранены без изменений. Лог `output/playwright/stage-5/final-qa.log`, итоговый вид `output/playwright/stage-5/final-main.png`.

При пересборке debug один раз возник os error 5 из-за запущенного собственного lunea.exe; процесс остановлен, последующие debug-сборки прошли. Это отличается от Code Integrity 4551 при последнем исполнении unit-test exe. Защита Windows не изменялась, заблокированный тест не выдаётся за прошедший.

Финальная release/NSIS-сборка после всех правок успешно завершена: `output/stage-5-release-complete.log`. Установщик `src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe`, 20 893 406 байт (~19,93 МиБ). Debug `output/stage-5-debug-verified.log` успешно собран и проверен в WebView2. Проверка unit-test процесса после последнего изменения всё ещё не подтверждена из-за описанной блокировки; повторного обхода/изменения политики не предпринималось. Перед передачей запущен release exe, Vite/debug остановлены; установка установщика не выполнялась.

Release smoke-check через native accessibility: настоящий трек «Буду Вуду», «Воспроизведение · GSMTC», кнопка Pause и обложка присутствуют на `http://tauri.localhost`. Нативная диагностика: один release-процесс, два Tauri окна, одна tray_icon_app, main видим. `output/playwright/stage-5/release-windows.json`.
