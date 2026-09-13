# ADR-003: HUD と設定の二窓

## Status

Accepted

## Context

オーバーレイ HUD にサイドバー等を載せると作業キャンバスを圧迫する。

## Decision

- **HUD 窓**: 透明・クリックスルー・キーボード図のみ
- **設定窓**: 通常ウィンドウ。`10-mockup-html` 必須装備はこちらに集約

## Consequences

- Overlay 中の操作はホットキー（`Ctrl+Shift+K`）とトレイ経由
- モックも同じ二画面構成で実装正本を揃える
