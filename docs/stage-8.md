# Этап 8 — фирменный установщик Windows

## Архитектура

Выбран один NSIS-пакет, custom template Tauri 2 + встроенные nsDialogs/MUI2.
Основное приложение, его темы, Windows Audio/GSMTC/Discord и permissions не менялись.
Идентичность сохранена: LUNÉA, app.lunea.audio, lunea.exe, 0.1.0, x64,
per-user/HKCU; RequestExecutionLevel user. Дополнительного bootstrapper,
Electron, runtime/npm/Rust зависимостей, сетевой телеметрии нет.

Вариант A позволяет оставить настоящий прогресс NSIS, штатную регистрацию,
ярлыки, WebView2 prerequisite и удаление. Вариант B мог бы дать CSS-анимации,
но потребовал бы второго процесса/пакета, верификации payload и защиты от TOCTOU,
обработки отмены и собственного обнаружения установленной версии; WebView2 UI
ещё и зависит от компонента, которого может не быть до установки. Для статичного
исходного изображения эти риски и стоимость не оправданы.

Основа закреплена в upstream.nsi: Tauri commit
d62e5d9c0878b6fb327543cc9be50aa1696ee23c; источник/лицензия рядом.
prepare-template.mjs применяет явные изменения и падает, если anchors upstream
поменялись. Не обновлять upstream автоматически при обновлении Tauri CLI.

Источники: [Tauri NSIS customization](https://v2.tauri.app/distribute/windows-installer/),
[NSIS nsDialogs](https://nsis.sourceforge.io/Docs/nsDialogs/Readme.html),
[NSIS MUI2](https://nsis.sourceforge.io/Docs/Modern%20UI%202/Readme.html).

## Маскот и оформление

- Использован только предоставленный C:/Users/riasg/Downloads/installer.png.
  Оригинал 1312×1199, 2,272,020 bytes, alpha extrema 0–255.
- Копия installer-original.png совпадает по SHA256:
  849814D2D5D87CEE1A55A242DF8FD9F67B9EE9B69833D3D3F1DF637D69838A80.
- Производные: PNG 900×822 с alpha, bitmap-композиция 600×668 и header 300×114.
  Только пропорциональное уменьшение и alpha composition на #0d1216, без обрезки,
  генерации, перекрашивания или изменения лица/волос/одежды. Восемь app PNG не тронуты.
- Тёмные native pages, Neon Pink, secondary #e19cbb, Windows 11 dark caption,
  настоящие клавиатурные controls. Токены оформления находятся в lunea.nsh.
  Остальные акценты документированы как значения токенов; переключателя тем нет.
- Из одного кадра не делались моргание, движения конечностей, поворот или взмах.
  Персонаж статичен на Welcome/Finish и в header остальных страниц; нет animation
  timers, canvas или движения всей картинки. Reduced motion соблюдается статичным UI.
  CSS/полупрозрачные web-панели и полноценные анимации не заявляются для native NSIS.

## Поведение

Welcome → обнаруженная установка (если есть) → реальный выбор папки → опции
desktop/Start Menu/run-after → настоящий InstFiles → подтверждённый Finish.
Запуск после установки opt-in (выключен по умолчанию); отдельная кнопка запуска
на Finish доступна по явному нажатию. Автозапуск Windows не включается.

Существующая HKCU установка обновляется в её зарегистрированной папке, включая
попытку передать другой /D. Предварительное удаление старой версии из штатной
maintenance page исключено: отмена подготовки не удаляет установленную копию.
Незарегистрированный lunea.exe/uninstall.exe не перезаписывается автоматически.
Downgrade запрещён; миграция с MSI/WiX не реализована этим custom template.

Новый exe сначала извлекается в .new.exe. Старый переименовывается в .previous.exe
и остаётся до .onInstSuccess. Ошибка замены пытается восстановить старый exe.
Проверяются создание uninstaller и запись регистрации. Предсуществующие staging
файлы не удаляются произвольно; cleanup касается только созданного данным запуском
staging. Это защита от обычных ошибок, НЕ полноценная транзакционная гарантия при
отключении питания/kill процесса; оставшийся .previous.exe требует диагностики.

До копирования есть штатное подтверждение отмены. На коротком критическом этапе
кнопка Cancel отключена, о чём предупреждено на странице опций; процесс принудительно
не убивается. Ошибка не ведёт на успешный Finish. Native error UI позволяет закрыть
установщик и повторить после устранения причины; встроенные File errors используют
NSIS Retry. Минимальный диагностический лог: %TEMP%/Lunea-setup-0.1.0.log.

Обычное обновление не удаляет APPDATA. Uninstaller сохраняет штатный unchecked
флажок явного удаления пользовательских данных. При обычном удалении без него
настройки сохраняются; при обновлении автозапуск не переписывается. При удалении
удаляется регистрация автозапуска данного продукта штатным NSIS-кодом Tauri.

## Изменённые файлы

- src-tauri/tauri.conf.json — custom NSIS config, currentUser, Russian, запрет downgrade.
- src-tauri/installer/upstream.nsi, UPSTREAM.txt, LICENSE-MIT — закреплённая база.
- src-tauri/installer/template.nsi — сгенерированный custom template, в репозитории.
- src-tauri/installer/lunea.nsh — native страницы, токены, диагностика/замена exe.
- src-tauri/installer/assets/* — оригинал и производные installer artwork + metadata.
- scripts/installer/prepare-assets.py, prepare-template.mjs — воспроизводимая подготовка.
- scripts/installer/build-qa.ps1 — отдельный QA product/identifier/registry, не production.
- docs/stage-8.md, README.md, context/CURRENT_STATE.md — результаты/продолжение.

## Проверки и безопасный сценарий

Логи: output/stage-8/. Typecheck, npm build, cargo check и 13/13 Rust выполнены.
Frontend source этапа 7 не менялся. Первая release-компиляция была успешна, затем
исправлена синтаксическая ошибка File /oname в NSIS; успешный bundle после исправления.
Финальный полный release build прошёл (release-final.log); последующие NSIS-only правки оформления скомпилированы успешно (nsis-final-verified.log).

Native UI: проверены Welcome, редактирование папки с пробелами, Browse/отмена
диалога выбора, опции, отмена подготовки. cancel-before-install.json: false для
регистрации QA и файла lunea.exe. Playwright не управляет native NSIS; использован
Computer Use. 21st UI review применён как ручной rubric по реальному окну;
автоматический HTML/CSS scanner не выдаётся за проверку NSIS controls.

Для real install/update/uninstall подготовлен LUNEA-Installer-QA с отдельными
MANUFACTURER, PRODUCTNAME, BUNDLEID и папкой output/stage-8/sandbox.
Production payload тот же, но QA не должен запускать app с пользовательскими
настройками. ObserveUI определяет только QA compile flag: 10-секундную паузу после
реальной распаковки для снимка прогресса и Finish при passive launch. В production
эти ветки не скомпилированы; процент не моделируется и не подменяется.

Автоматическая проверка отклонила запуск QA-установки: «blocked by policy».
Пользователь ответил «Нет, только собрать установщик». Реальные install/update/uninstall НЕ выполнялись; подтверждение больше не ожидается, эти сценарии исключены по его последнему указанию.
Пользовательская установленная копия не перезаписывалась и не удалялась.

## Перед распространением

Нужны легитимная code signing/доверенная дистрибуция, чистая Windows VM с отсутствующим
WebView2, реальные DPI 100/150% (текущий 125%), install/update/uninstall matrix,
файл-in-use/недостаток места/ошибки доступа и проверка отсутствия данных после explicit
data-delete opt-in. Не обходить CI/Smart App Control/UAC; unsigned статус не маскировать.
Не менять version/identifier ради теста production копии. Следующий этап не запускался.

## Итоговый пакет · сборка без установки

По последнему указанию пользователя работа ограничена сборкой. Реальной установки
(в том числе QA), обновления и удаления не было. QA registry и production registry
не созданы. APPDATA JSON совпадают по хешам с состоянием до installer QA:
output/stage-8/settings-preserved.json, equal=true. Основное приложение возвращено
к обычному запуску без диагностического CDP; внешние аудиоклиенты не переключались.

Готовый файл: src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe.
Размер **21,447,874 bytes (20.45 MiB)**. SHA256:
`954A434E7420774F1D8EDE53254DEE424534D0BDF9135692231AE36732CCEB1F`.
Authenticode: **NotSigned**. Финальные метаданные: output/stage-8/installer.json.
Версия и bundle identifier сохранены. Старый Stage 7 setup сохранён отдельно
в output/stage-8/stage7-setup.exe; оригинальный mascot Downloads не изменён.

Результаты: npm typecheck/build, cargo check, cargo test --lib 13/13,
frontend regression 11/11, полная release-компиляция и NSIS bundle пройдены.
В процессе исправлены File /oname syntax, runtime handles DirectoryShow,
тёмный фон native pages/checkboxes, ширина CTA, Segoe UI 10pt, header placement.
После повторного bundle уже маркированного exe CLI иногда предупреждал об отсутствии
__TAURI_BUNDLE_TYPE marker; полная release-сборка прошла штатно. Updater plugin
в проект не добавлен, его обновления этим этапом не проверялись.

Playwright smoke настоящего release WebView2: Audio ready, Media paused,
Discord ready + mute/deafen null, Ctrl+D registered; main/overlay без горизонтального
overflow и broken images (main-smoke.log). Это проверка рабочего приложения,
а не установка пакета. Новая матрица четырёх тем не повторялась: frontend не менялся,
матрица этапа 7 сохранена в его отчёте. Native NSIS проверен при текущем DPI 125%;
100/150% физически не переключались, Playwright emulation не применяется к NSIS.

Окончательный screenshot Welcome: output/stage-8/welcome.png. Также нативно
просмотрены Directory, Browse и Options. Экран копирования/Finish/ошибки установки
вживую НЕ проходились — установка не разрешена; скриншоты этих экранов не подделывались.
Старые QA-снимки не выдаются за финальный production интерфейс.

Для ручного запуска готового файла откройте его двойным щелчком; после Welcome
проверьте папку и опции. Установка начнётся только на кнопке «Установить».
Выполнять это сейчас автоматически не требуется. До распространения нужны подпись
и перечисленные выше установочные проверки на безопасной машине. Следующий этап
не запускался. Сборочная часть этапа 8 завершена; установка/обновление/удаление
остаются непроверенными по решению пользователя.
