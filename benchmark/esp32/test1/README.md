# ESP32 Benchmark Test1

Этот benchmark тестирует два одновременных процесса на ESP32:
1. **Задача счётчика**: инкрементирует счётчик на 1024 в цикле
2. **Задача таймера**: печатает значение аппаратного таймера

Обе задачи работают в рамках cooperative планировщика задач Martos.

## Структура

Программа содержит две задачи, добавленные в cooperative планировщик задач:

- `counter_task_*`: Функции для задачи счётчика, которая инкрементирует AtomicU32 на 1024 в каждой итерации. Задача останавливается после достижения значения 100000.
- `timer_task_*`: Функции для задачи таймера, которая получает и выводит значение аппаратного таймера ESP32. Задача работает бесконечно.

## Установка зависимостей

Для установки зависимостей для разработки приложений под архитектуру Xtensa ESP32,
см. [официальный сайт](https://docs.esp-rs.org/book/installation/riscv-and-xtensa.html).

Пример установки на Linux (Ubuntu/Debian):
```bash
apt-get -qq update
apt-get install -y -q build-essential curl
curl https://sh.rustup.rs -sSf | sh -s -- -y
cargo install espup
espup install
```

## Сборка

Установите переменные окружения ESP32:
```bash
. $HOME/export-esp.sh
```

Затем соберите проект:
```bash
cargo build --release
```

## Запуск

Для подробных инструкций по запуску проектов для ESP32 см. [официальный сайт](https://docs.esp-rs.org/book/tooling/espflash.html).

```bash
cargo run
```
