# KeyLayerHUD — QMK ファーム小片

VIA 標準プロトコルには**実行中レイヤー**取得コマンドが無いため、Raw HID で最小報告を追加します。

## 手順

1. `rules.mk` に `RAW_ENABLE = yes`（VIA 機は既に有効なことが多い）
2. `layer_report.c` を keymap ディレクトリへコピー、または `#include "layer_report.c"` を `keymap.c` 末尾に追加
3. `keymap.c` で `raw_hid_receive_kb` を実装している場合は `klh_raw_hid_receive` を呼ぶ

```c
void raw_hid_receive_kb(uint8_t *data, uint8_t length) {
    klh_raw_hid_receive(data, length);
}
```

## プロトコル

| バイト | 意味 |
|--------|------|
| 0 | `0x42` クエリ / `0x43` プッシュ報告 |
| 1 | `get_highest_layer(layer_state)` |
| 2-5 | `layer_state` (LE32) |
| 6 | `get_mods()` |

コマンド `0x42` は VIA の `0x01`〜`0x16` と衝突しません。

## 注意

- KeyLayerHUD 接続中は VIA/VIAL GUI を閉じてください（Raw HID 排他）
- 無線のみ ZMK（USB 無し）は本 MVP 対象外
