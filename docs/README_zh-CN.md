# git-simple-encrypt

[English](../README.md) | 简体中文

这是一个非常简单易用的 git 部分文件加密工具，只需一个密码，即可在任何设备上加密/解密您的 git 仓库。

- 为什么不使用 [`git-crypt`](https://github.com/AGWA/git-crypt)？
  - **比起安全性，本项目更加注重易用性。** 单密码对称加密是我的底线。

> [!WARNING]  
> **本仓库不对加密安全性与向后兼容性作出任何保证。** （请使用同一个大版本）

## 安装

有几种不同方法可以安装此程序，您可以选择其中**任意一种**。

- 在 [Releases](https://github.com/lxl66566/git-simple-encrypt/releases) 中下载文件并解压，放入 `C:\Windows\System32`（如果您用的是 windows）或任意 `Path` 目录下。
- 使用 [bpm](https://github.com/lxl66566/bpm)：
  ```sh
  bpm i git-simple-encrypt -b git-se -q
  ```
- 使用 [scoop](https://scoop.sh/)：
  ```sh
  scoop bucket add absx https://github.com/absxsfriends/scoop-bucket
  scoop install git-simple-encrypt
  ```
- 使用 cargo：
  ```sh
  cargo +nightly install git-simple-encrypt
  ```

## 使用

```sh
git se set 123456       # 设置密码为 `123456`
git se e                # 加密当前仓库所有列表内的文件
git se d                # 解密...
git se add file.txt     # 将 `file.txt` 添加到加密列表
```

## 原理

```mermaid
graph TD;
    123 -- SHA3_224 --> 602bdc204140db016bee5374895e5568ce422fabe17e064061d80097 -- CUT --> 602bdc204140db016bee5374895e5568 --cipher--> Aes128GcmSiv  -- output--> b5e9ecbf3efaad4380998b1379d679a39109e150df74
    123456 --content--> Aes128GcmSiv
    CONST --NONCE--> Aes128GcmSiv
```