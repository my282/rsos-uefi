# ロードマップ

参照: [phil-opp] = Writing an OS in Rust 2nd ed / [みかん] = ゼロからのOS自作入門

## Phase 0: 土台

- [x] ~~Cargo ワークスペース作成 (`loader`, `kernel`)~~
      →修正。`loader`と`kernel`は別々のクレートに(decisions.md参照)
- [x] ビルド → ESP 作成 → QEMU+OVMF 起動を 1 コマンドで行うスクリプト
- [x] QEMU のシリアル出力をターミナルに出す (`-serial stdio`)

## Phase 1: UEFI ローダ [みかん 1〜4章 / uefi-rs]

- [x] ローダで画面とログに出力 (uefi-tutorial の復習)
- [x] メモリマップの取得・表示 [みかん 2.3〜2.7]
- [ ] GOP でフレームバッファ情報取得、塗りつぶし [みかん 3.4]
- [ ] カーネルを ESP から読み込んでジャンプ [みかん 3.3]
- [ ] カーネル ELF の PT_LOAD セグメントを正しく配置 [みかん 4.5]
- [ ] ブート情報構造体 (フレームバッファ, メモリマップ) の設計 → `docs/decisions.md`
- [ ] ExitBootServices → カーネルのエントリへジャンプ [みかん 3.3, 3.6 エラー処理]
- [ ] カーネル側で画面を塗りつぶせたら Phase 1 完了 [みかん 3.5]

## Phase 2: 出力まわり [phil-opp: VGA Text Mode の置き換え]

- [ ] シリアルポート (UART 16550) 出力、`serial_println!`
- [ ] フレームバッファへのピクセル描画 (ピクセル形式の抽象化) [みかん 4.2〜4.3]
- [ ] フォント描画と `println!` 相当のコンソール [みかん 5章]
- [ ] panic 時にメッセージ表示

## Phase 3: テスト [phil-opp: Testing]

- [ ] custom_test_frameworks or 代替手段の検討 (stable で行けるか)
- [ ] `isa-debug-exit` で QEMU を終了させる統合テスト

## Phase 4: CPU 例外 [phil-opp: CPU Exceptions, Double Faults]

- [ ] GDT / TSS を自前で設定 (UEFI のものから切り替え) [みかん 8.5]
- [ ] IDT とブレークポイント例外 [みかん 7.1〜7.4]
- [ ] ダブルフォルト + IST

## Phase 5: ハードウェア割り込み [phil-opp: Hardware Interrupts]

- [ ] 8259 PIC でタイマ割り込み (まず記事通り)
- [ ] PS/2 キーボード入力
- [ ] (独自) APIC へ移行 [みかん 7章 (MSI/Local APIC), 11.2]

## Phase 6: メモリ管理 [phil-opp: Paging ×2, Heap Allocation, Allocator Designs]

- [ ] UEFI のメモリマップを使ったフレームアロケータ [みかん 8.2, 8.7]
- [ ] スタックとローダのデータを UEFI 管理領域から移す [みかん 8.3〜8.4]
- [ ] 自前ページテーブル構築 (物理メモリ全体のオフセットマップ) [みかん 8.6]
- [ ] ヒープ + `alloc` クレート
- [ ] アロケータ自作 (bump / linked list / fixed size)

## Phase 7: 非同期 [phil-opp: Async/Await]

- [ ] 協調的マルチタスク (executor, キーボードタスク)

## Phase 8 以降: 独自要素 (候補)

- ACPI テーブル解析 / APIC タイマ / ACPI PM タイマ [みかん 11〜12章]
- プリエンプティブなスケジューラ・スレッド [みかん 13〜14章]
- ユーザーモード・システムコール [みかん 後半]
- PCI 列挙、xHCI(USB) などのドライバ [みかん 6章, 12.3]
- ファイルシステム (FAT) [みかん 後半]
- ウィンドウシステム・ターミナル [みかん 9〜10, 15〜16章]
- 実機ブート
