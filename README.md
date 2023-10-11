# embassy-template

Note: If you are writing code for STM32F303RE, clone the already-made [f303 branch](https://github.com/McMaster-Rocketry-Team/embassy-template/tree/main-f303).

## Cloning

```shell
git clone --recurse-submodules https://github.com/McMaster-Rocketry-Team/embassy-template.git
```

## How to Change Application Name

- Line 5 & 32 in `Cargo.toml`
- Line 17 in `.vscode/launch.json`
- Line 2 in `.cargo/config.toml`

## How to Change STM32 Model

- Line 2 in `Embed.toml`
- Line 12 in `Cargo.toml`
- Line 12 in `.cargo/config.toml`
- Update `FLASH_SIZE_KIB` and `RAM_SIZE_KIB` in `size.py`
- Add svd file for the model to `.vscode/`
- Line 18 in `.vscode/launch.json`

## Start

```bash
cargo run
```

## Calculate Size

```bash
./size.py
```
