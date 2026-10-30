## Phase 0: 土台
> - [x] ~~Cargo ワークスペース作成 (`loader`, `kernel`)~~ 
>       →修正。`loader`と`kernel`は別々のクレートに(decisions.md参照)

loaderとkernelでターゲットが違い、完全に分割してしまったほうが管理が楽。
(config.tomlやrust-toolchain.tomlなど)

また、その影響で.lazy.luaを作成。個人的なエディタ設定ではあるが、一応コミットしておく。

## QEMU の起動ディスクを vvfat からディスクイメージに変更

`-drive file=fat:rw:esp`（QEMU の vvfat）では、ゲストからのファイル書き込みがホスト側に
正しく反映されなかった（ファイルを作り直してもサイズが更新されず、古い内容が残る）。
最終的に実機で起動したいので、実機の USB メモリに近い FAT32 のディスクイメージ（`disk.img`）を
`mkfs.fat` + `mtools` で毎回作り直して起動する形にした。sudo での mount は不要。
