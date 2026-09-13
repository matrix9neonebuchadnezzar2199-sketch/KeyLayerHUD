# ADR-002: ライブレイヤーはファーム小片

## Status

Accepted

## Context

VIA 標準コマンド `0x01`〜`0x16` はキーマップ読み出しのみで、実行中 `layer_state` は取得できない。

## Decision

`firmware/qmk/layer_report.c` で Raw HID `0x42` クエリ / `0x43` プッシュを実装する。

## Consequences

- ユーザーは短いファーム追加が必要（VIAL issue #310 相当のワークアラウンド）
- OS キーフックによる推定は採用しない（二重設定・ズレ回避）
