# KeyLayerHUD

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS-lightgrey)](README.md)
[![Repo](https://img.shields.io/badge/GitHub-Public-green)](https://github.com/matrix9neonebuchadnezzar2199-sketch/KeyLayerHUD)
[![Stack](https://img.shields.io/badge/Stack-Tauri%202%20%2B%20Rust%20%2B%20TS-orange)](README.md)
[![Status](https://img.shields.io/badge/Status-MVP%20Phase%20A-yellow)](README.md)

分割・自作キーボードの**現在アクティブなレイヤーとキーマップ**を、画面最前面に半透明で常時表示する HUD ツール。

## 目次

- [概要](#概要)
- [読者別導線](#読者別導線)
- [動作環境](#動作環境)
- [インストール](#インストール)
- [使用例](#使用例)
- [終了](#終了)
- [トラブルシューティング](#トラブルシューティング)
- [ライセンス](#ライセンス)
- [Topics](#topics)

## 概要

- **HUD 窓**: 透明・常時最前面・クリックスルー（Overlay モード）
- **設定窓**: 接続・レイアウト・キーマップ・HUD 表示・終了
- **Mock モード**: キーボード未着でも Corne 42 相当サンプルで動作確認可能
- **実機**: VIA キーマップ読み出し + ファーム小片によるライブレイヤー（`firmware/qmk/`）

## 読者別導線

| 読者 | 最初に読む |
|------|------------|
| 利用者 | [インストール](#インストール) → [使用例](#使用例) |
| ファーム改修者 | [firmware/qmk/README.md](firmware/qmk/README.md) |
| 他端末・実装 | [docs/設計書.md](docs/設計書.md) → [docs/設計図.md](docs/設計図.md) → [docs/adr/](docs/adr/) |
| 検証 | [TEST.py](TEST.py) |

## 動作環境

- Windows 11 / macOS 12+（開発・検証は Windows 11）
- Node.js 20+
- Rust stable（`rustup`）
- QMK + VIA/VIAL 互換キーボード（実機連携時）

## インストール

### 1. クローン

```powershell
git clone https://github.com/matrix9neonebuchadnezzar2199-sketch/KeyLayerHUD.git
cd KeyLayerHUD
```

### 2. 依存関係

```powershell
npm install
```

### 3. 開発起動

```powershell
npm run tauri dev
```

### 4. ブラウザ Mock（Tauri なし）

```powershell
npm run dev
```

ブラウザで `http://localhost:1420/hud.html?mock=1` を開く。

## 使用例

| 操作 | 方法 |
|------|------|
| Overlay ↔ Edit | `Ctrl+Shift+K` または HUD 上のボタン |
| 設定を開く | トレイ → 設定 / トレイ左クリック |
| Mock レイヤー切替 | 設定 → 接続モード Mock、HUD 下部タブ |
| キーマップ JSON | 設定 → キーマップ → インポート |
| ファーム追加 | `firmware/qmk/layer_report.c` を keymap に組み込み |

## 終了

- **HUD を閉じるだけでは終了しません**（トレイ常駐）
- 完全終了: トレイ → **終了**、または設定 → 終了

## トラブルシューティング

| 症状 | 原因 | 対処 |
|------|------|------|
| VIA と同時に繋がらない | Raw HID 排他 | VIA/VIAL GUI を閉じる（[ADR-001](docs/adr/001-hid-exclusive.md)） |
| レイヤーが切り替わらない | 標準 VIA にライブレイヤー無し | `firmware/qmk/` を適用 |
| HUD がクリックを奪う | Overlay モード | `Ctrl+Shift+K` で Edit に切替 |
| デバイス未検出 | VID/PID 未設定 / ケーブル | Mock モードで HUD を先に確認 |

## ライセンス

MIT — Copyright (c) 2026 Hide-Infa

## Topics

`keyboard` `qmk` `via` `vial` `split-keyboard` `overlay` `tauri` `hud` `keymap`
