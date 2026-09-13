# ADR-001: VIA Raw HID 排他

## Status

Accepted

## Context

VIA / VIAL GUI と KeyLayerHUD は同じ Usage Page `0xFF60` の Raw HID エンドポイントを使う。

## Decision

HUD 接続中は VIA/VIAL アプリを閉じる運用とし、同時接続はサポートしない。

## Consequences

- ユーザーはキーマップ編集とライブ HUD を同時に使えない
- 実装は単一 HID ハンドルで十分（競合エラーを UI に表示）
