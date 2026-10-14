# VoiceFlow

Личное macOS-приложение для диктовки: удерживаешь `Fn` — говоришь — текст сам вставляется
туда, где стоит курсор. Транскрипция через облачные провайдеры по своим API-ключам.

Форк [Dictara](https://github.com/vitalii-zinchenko/dictara), переработанный под себя.
Планы по перестройке — в [REBUILD.md](REBUILD.md).

## Стек

- **Ядро:** Rust + Tauri v2
- **UI:** React 19 + TypeScript + Tailwind + shadcn/ui
- **Мост:** `tauri-specta` → типизированные биндинги в `src/bindings.ts`
- **Секреты:** macOS Keychain

## Провайдеры

OpenAI · Azure OpenAI · OpenRouter · ElevenLabs · любой OpenAI-совместимый endpoint

## Разработка

```bash
npm install
npm run dev:tauri      # запуск приложения в дев-режиме
npm run verify         # типы, форматирование, clippy, сборка
```

Логи: `.logs/app.log` в дев-режиме, `~/Library/Logs/app.voiceflow/VoiceFlow.log` в релизе.

## Fn-клавиша и эмодзи-пикер

Если по `Fn` открывается эмодзи-пикер, поменяй в **System Settings → Keyboard →
«Press 🌐 key to»** на **Do Nothing**. Это системное поведение macOS, перехватить его
из приложения нельзя.

## Лицензия

MIT — см. [LICENSE](LICENSE).
