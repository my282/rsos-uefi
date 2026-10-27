# rsos-uefi

Rust 製の x86_64 UEFI 向け自作 OS。
「Writing an OS in Rust」(https://os.phil-opp.com/ 第2版) の内容を UEFI 前提で実装し、その後は独自機能を追加していく。

## 進め方（重要）
- **コードを書くのはユーザー本人。** Claude はメンター役。
  - 概念の説明、次の一手のヒント、参照すべき資料・仕様の案内、コードレビュー、デバッグ支援を行う。
  - 実装コードは明示的に頼まれたときだけ書く。ヒントは短い断片や疑似コードに留める。
  - CLAUDE.md・docs・ビルド/実行スクリプトなどの周辺ファイルは提案のうえ用意してよい。
- 進捗は `docs/roadmap.md` のチェックリストで管理する。区切りごとにコミットし、ロードマップを更新する。
- 設計上の決定は `docs/decisions.md` に理由とともに残す（なければ作る）。

## 構成（予定）
- `loader/` : UEFI ブートローダ（`x86_64-unknown-uefi`, `uefi` クレート）
- `kernel/` : カーネル本体（`x86_64-unknown-none`）
- ビルド〜QEMU 起動スクリプト（xtask か run.sh、未定）

ローダの責務: メモリマップ取得 → カーネル ELF ロード → GOP フレームバッファ情報取得 → ExitBootServices → ブート情報を渡してカーネルへジャンプ。

## 環境
- Arch Linux (Omarchy), Rust stable (rustup)
- QEMU: `qemu-system-x86_64`
- OVMF: `/usr/share/edk2/x64/OVMF_CODE.4m.fd`, `OVMF_VARS.4m.fd`（VARS は書き込まれるのでコピーして使う）

## 参考資料
- **`docs/uefi-reference.md`**: UEFI 仕様 2.11 の要点まとめ（用語・ルール・定数。各項目に仕様書の章番号付き）。UEFI の仕様で迷ったらまずここを見る
- Writing an OS in Rust (2nd ed): https://os.phil-opp.com/ — BIOS 前提なので VGA text mode / bootloader 0.9 部分は読み替える
- 『ゼロからのOS自作入門』(みかん本) PDF: `~/Documents/mikanos.pdf` (749ページ) — UEFI ローダ部分 (1〜4章) の主要参考書。PDF のページ番号 = 本のページ番号 + 2。必要な章だけ `pdftotext -f N -l M` で読む
- uefi-rs: https://docs.rs/uefi / https://github.com/rust-osdev/uefi-rs
- `bootloader` クレート 0.11 のソース (UEFI ローダの Rust 実装例、答え合わせ用)
- UEFI 仕様書, OSDev Wiki
- 練習用: `../uefi-tutorial/`（別リポジトリ、uefi-rs の Hello world）
