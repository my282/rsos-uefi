## Phase 0: 土台
> - [x] ~~Cargo ワークスペース作成 (`loader`, `kernel`)~~ 
>       →修正。`loader`と`kernel`は別々のクレートに(decisions.md参照)

loaderとkernelでターゲットが違い、完全に分割してしまったほうが管理が楽。
(config.tomlやrust-toolchain.tomlなど)

また、その影響で.lazy.luaを作成。個人的なエディタ設定ではあるが、一応コミットしておく。
