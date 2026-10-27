# UEFI 開発者向けリファレンス

UEFI アプリケーション（特に OS ローダ）を書くときに必要な用語・仕様上のルールをまとめた手元用リファレンス。
ファームウェア側の実装（EDK II の内部構造など）は扱わない。

- **一次出典**: UEFI Specification **2.11**（UEFI Forum, <https://uefi.org/specs/UEFI/2.11/>）
- **補助出典**: EDK II `MdePkg/Include`（UEFI Forum 参加企業が保守する仕様準拠ヘッダ。定数値・GUID の確認に使用）、PI Specification（ブートフェーズの説明のみ）
- 各項目の末尾の `[§x.y]` は UEFI 2.11 の章番号。詳細を確認したいときはそこを引く。
- 型名・関数名は仕様書の表記（C 言語風）のまま書く。

---

## 目次

1. [全体像](#1-全体像)
2. [基本データ型](#2-基本データ型)
3. [x64 の実行環境と呼び出し規約](#3-x64-の実行環境と呼び出し規約)
4. [UEFI イメージ](#4-uefi-イメージ)
5. [System Table](#5-system-table)
6. [ハンドルとプロトコル](#6-ハンドルとプロトコル)
7. [Boot Services 一覧](#7-boot-services-一覧)
8. [TPL とイベント](#8-tpl-とイベント)
9. [メモリ管理](#9-メモリ管理)
10. [ExitBootServices](#10-exitbootservices)
11. [Runtime Services と UEFI 変数](#11-runtime-services-と-uefi-変数)
12. [ブートマネージャ](#12-ブートマネージャ)
13. [主要プロトコル](#13-主要プロトコル)
14. [ディスク: GPT と ESP](#14-ディスク-gpt-と-esp)
15. [Configuration Table](#15-configuration-table)
16. [Secure Boot の概要](#16-secure-boot-の概要)
17. [ステータスコード](#17-ステータスコード)
18. [落とし穴チェックリスト](#18-落とし穴チェックリスト)
19. [用語集](#19-用語集)
20. [参考資料](#20-参考資料)

---

## 1. 全体像

### UEFI と周辺仕様

| 仕様 | 内容 | OS ローダ開発者との関係 |
|---|---|---|
| **UEFI Specification** | ファームウェアと OS/アプリの**インタフェース**（System Table、Boot/Runtime Services、プロトコル、ブートマネージャ等） | これを読む |
| **PI Specification** (Platform Initialization) | ファームウェアの**内部構造**（SEC/PEI/DXE などのフェーズ、ドライバの実行方式） | 基本的に不要 |
| **ACPI Specification** | ハードウェア構成・電源管理の記述テーブル | OS 側で必要（RSDP は UEFI の Configuration Table から得る） |

UEFI 仕様は「ファームウェアが何を提供するか」だけを定め、どう実装するかは定めない。EDK II（TianoCore）はその代表的な実装で、QEMU 用の OVMF も EDK II 製。

### ブートの流れ（PI 仕様のフェーズ名）

```
SEC → PEI → DXE → BDS → TSL → RT → AL
                   │      │     │
                   │      │     └ Runtime: OS 稼働中（Runtime Services のみ）
                   │      └ Transient System Load: OS ローダが動く期間
                   └ Boot Device Selection: ブートマネージャがブート対象を選ぶ
```

- **SEC / PEI**: CPU・メモリの初期化。アプリから見えない。
- **DXE** (Driver Execution Environment): ドライバがロードされ、Boot Services とプロトコルが揃う。
- **BDS**: ブートマネージャが `BootOrder` などの変数を見て `\EFI\BOOT\BOOTX64.EFI` などを起動する。
- **TSL**: 自作 OS ローダが動くのはここ。`ExitBootServices()` を呼ぶと RT へ移る。
- **AL** (After Life): リセット・シャットダウン後。

### 2 つの「時間」

UEFI 仕様は時間を 2 つに分ける。どのサービスが使えるかはこれで決まる。

| 期間 | 使えるもの |
|---|---|
| **Boot Services 時間**（`ExitBootServices()` 成功まで） | Boot Services、Runtime Services、すべてのプロトコル |
| **Runtime**（`ExitBootServices()` 成功後） | Runtime Services のみ |

[§2.2, §7]

---

## 2. 基本データ型

特記がない限り**自然アラインメント**（構造体は最大メンバのサイズ境界に揃い、内部はパディングされる）。UEFI に渡すポインタも自然アラインメントでなければならず、外れていると結果は不定。[§2.3.1]

| 型 | 意味 | x64 での実体 |
|---|---|---|
| `BOOLEAN` | 0 = FALSE, 1 = TRUE。それ以外は未定義 | 1 バイト |
| `INTN` / `UINTN` | ネイティブ幅の整数 | 8 バイト |
| `INT8`〜`UINT64` | 固定幅整数 | — |
| `CHAR8` | ASCII（ISO-Latin-1） | 1 バイト |
| `CHAR16` | **UCS-2** 文字 | 2 バイト |
| `VOID` | 型なし | — |
| `EFI_GUID` | 128 ビットの識別子。64 ビット境界にアライン | 16 バイト |
| `EFI_STATUS` | ステータスコード | `UINTN` |
| `EFI_HANDLE` | インタフェースの集合への不透明ポインタ | `VOID *` |
| `EFI_EVENT` | イベントへの不透明ポインタ | `VOID *` |
| `EFI_LBA` | 論理ブロックアドレス | `UINT64` |
| `EFI_TPL` | タスク優先度 | `UINTN` |
| `EFI_PHYSICAL_ADDRESS` | 物理アドレス | `UINT64` |
| `EFI_VIRTUAL_ADDRESS` | 仮想アドレス | `UINT64` |
| 列挙型 | `INT32` または `UINT32` | 4 バイト |

**文字列は基本的に UCS-2（`CHAR16`）の NUL 終端**。UTF-16 のサロゲートペアは使えない（FAT のファイル名も UCS-2 の範囲でしか扱わない [§13.3]）。

### 関数引数の修飾子

| 修飾子 | 意味 |
|---|---|
| `IN` | 呼び出し側 → 関数 |
| `OUT` | 関数 → 呼び出し側 |
| `OPTIONAL` | NULL を渡してよい |
| `CONST` | 読み取り専用 |
| `EFIAPI` | UEFI 呼び出し規約に従う関数（§3） |

### 表記上の注意: GUID のバイト順

`EFI_GUID` は `{UINT32, UINT16, UINT16, UINT8[8]}` の構造体で、前の 3 フィールドは**リトルエンディアン**で格納される。文字列表記 `8868E871-E4F1-11D3-BC22-0080C73C8881` をそのままバイト列にすると一致しないので注意。

---

## 3. x64 の実行環境と呼び出し規約

### Boot Services 時間の CPU 状態 [§2.3.4]

ファームウェアがアプリに制御を渡すとき、以下が保証される。

| 項目 | 状態 |
|---|---|
| プロセッサ | **ユニプロセッサ**（BSP だけが動いている） |
| モード | **ロングモード（64 ビット）** |
| ページング | **有効**。UEFI メモリマップに載っている領域は**アイデンティティマップ**（仮想 = 物理）。ただし保護のため R/W/X の一部が落とされている領域もある。メモリマップ外の領域のマッピングは未定義 |
| セグメント | フラットに設定済みで、それ以外には使われない |
| 割り込み | **有効**。ただしファームウェアが扱うのは Boot Services のタイマだけ（デバイスドライバはポーリングで動く） |
| `RFLAGS.DF` | クリア |
| スタック | **128 KiB 以上**、16 バイトアライン。スタックは実行不可にされている場合あり |
| x87 制御ワード | `0x037F` |
| MXCSR | `0x1F80` |
| `CR0.EM` / `CR0.TS` | 0（つまり SSE をそのまま使える） |

### エントリ時のレジスタ [§2.3.4.1]

| レジスタ | 中身 |
|---|---|
| `RCX` | `EFI_HANDLE ImageHandle` |
| `RDX` | `EFI_SYSTEM_TABLE *SystemTable` |
| `RSP` | リターンアドレスを指す |

### 呼び出し規約（`EFIAPI`）[§2.3.4.2]

x64 の `EFIAPI` は **Microsoft x64 呼び出し規約**と同じ。System V ABI（Linux の通常の規約）ではない。Rust では `extern "efiapi"` がこれに対応する。

- 整数引数 1〜4 番目: `RCX`, `RDX`, `R8`, `R9`。5 番目以降はスタック。
- 浮動小数点引数 1〜4 番目: `XMM0`〜`XMM3`。
- 呼び出し側は**シャドウスペース**（32 バイト）を確保する。
- 戻り値: 64 ビットに収まるものは `RAX`。
- 揮発（呼び出しで壊れる）: `RAX`, `RCX`, `RDX`, `R8`〜`R11`, `XMM0`〜`XMM5`
- 不揮発（呼ばれた側が保存）: `RBX`, `RBP`, `RDI`, `RSI`, `R12`〜`R15`, `XMM6`〜`XMM15`
- 呼び出し時のスタックは常に 16 バイトアライン。

### 自前のページテーブル / GDT / IDT を使う場合 [§2.3.4.3]

Boot Services 時間中にアプリが CR3・GDT・IDT を差し替えた場合、ファームウェアを呼ぶ前（およびタイマ割り込みでファームウェアが割り込んでくる可能性があるとき）にはファームウェア用の環境に戻す責任がアプリ側にある。現実的には「**差し替えは `ExitBootServices()` 後に行う**」か「差し替え中は割り込みを禁止する」。

### Runtime Services を呼ぶときの条件 [§2.3.4]

- ロングモード、ページング有効
- `SetVirtualAddressMap()` を呼んでいなければ、ランタイム領域が仮想 = 物理でマップされていること
- `RFLAGS.DF` クリア、**4 KiB 以上**のスタック（16 バイトアライン）
- x87 制御ワード / MXCSR / `CR0.EM` / `CR0.TS` は Boot Services 時と同じ条件
- 割り込みは有効・無効どちらでもよい

---

## 4. UEFI イメージ

### 形式 [§2.1.1]

UEFI の実行ファイル（**UEFI イメージ**）は **PE32+**（Windows の 64 ビット実行形式）のサブセット。ELF ではない。

- ファイル先頭は `MZ`、PE ヘッダは `PE\0\0`
- x64 の Machine 値は `0x8664`
- ファームウェアは `LoadImage()` で全セクションをメモリに読み込み、**リロケーションを適用**してから `AddressOfEntryPoint` を呼ぶ。そのため配置アドレスは事前に分からない（PIE 相当）。

### Subsystem によるイメージの種類

| Subsystem 値 | 種類 | コードのメモリタイプ | データのメモリタイプ | 終了時 |
|---|---|---|---|---|
| 10 `EFI_APPLICATION` | **アプリケーション**（OS ローダはこれ） | `EfiLoaderCode` | `EfiLoaderData` | エントリから戻ると**必ずアンロード** |
| 11 `EFI_BOOT_SERVICE_DRIVER` | Boot Service ドライバ | `EfiBootServicesCode` | `EfiBootServicesData` | エラーを返したときだけアンロード（成功なら常駐） |
| 12 `EFI_RUNTIME_DRIVER` | Runtime ドライバ | `EfiRuntimeServicesCode` | `EfiRuntimeServicesData` | 同上。`ExitBootServices()` 後も残る |

**UEFI OS ローダ**は特別な種類ではなく、「`ExitBootServices()` を呼んでシステムを引き継ぐ UEFI アプリケーション」のこと。[§2.1.3]

### エントリポイント [§4.1, §7.4.4]

```c
typedef EFI_STATUS (EFIAPI *EFI_IMAGE_ENTRY_POINT)(
    IN EFI_HANDLE        ImageHandle,   // 自分自身を表すハンドル
    IN EFI_SYSTEM_TABLE *SystemTable    // すべての入口
);
```

- `ImageHandle` には必ず **Loaded Image Protocol** と **Loaded Image Device Path Protocol** が付いている。ここから「自分がどのデバイスのどのファイルから読まれたか」が分かる。
- エントリから `return` すると、その戻り値で `Exit()` を呼んだのと同じ扱い。
- アプリは終了前に、自分で確保したもの（メモリ・開いたファイルなど）をすべて解放する責任がある。[§7.2, §7.4.5]

---

## 5. System Table

すべての UEFI 機能への入口。エントリポイントで受け取る。[§4.3]

```c
typedef struct {
  EFI_TABLE_HEADER                 Hdr;
  CHAR16                          *FirmwareVendor;      // 例: L"EDK II"
  UINT32                           FirmwareRevision;
  EFI_HANDLE                       ConsoleInHandle;
  EFI_SIMPLE_TEXT_INPUT_PROTOCOL  *ConIn;
  EFI_HANDLE                       ConsoleOutHandle;
  EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *ConOut;
  EFI_HANDLE                       StandardErrorHandle;
  EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL *StdErr;
  EFI_RUNTIME_SERVICES            *RuntimeServices;
  EFI_BOOT_SERVICES               *BootServices;
  UINTN                            NumberOfTableEntries;
  EFI_CONFIGURATION_TABLE         *ConfigurationTable;  // ACPI, SMBIOS など (§15)
} EFI_SYSTEM_TABLE;
```

`ExitBootServices()` 成功後、`ConsoleInHandle`, `ConIn`, `ConsoleOutHandle`, `ConOut`, `StandardErrorHandle`, `StdErr`, `BootServices` は **NULL にされる**（CRC も再計算される）。[§7.4.6]

### テーブルヘッダ [§4.2]

System Table / Boot Services Table / Runtime Services Table の先頭に付く。

```c
typedef struct {
  UINT64 Signature;   // "IBI SYST" / "BOOTSERV" / "RUNTSERV"
  UINT32 Revision;    // 上位16bit = メジャー、下位16bit = マイナー
  UINT32 HeaderSize;  // ヘッダを含むテーブル全体のサイズ
  UINT32 CRC32;       // このフィールドを 0 にして HeaderSize バイト分を計算
  UINT32 Reserved;
} EFI_TABLE_HEADER;
```

- **リビジョンの読み方**: `(2 << 16) | 110` → 2.11、`(2 << 16) | 100` → 2.10、`(2 << 16) | 31` → 2.3.1。マイナー値を 10 で割った商と余り。
- テーブルは将来拡張されうるので、サイズは `sizeof` ではなく **`HeaderSize`** を使う。機能の有無は `Revision` が導入時のバージョン以上かで判断する。
- CRC は CCITT32（多項式 `0x04C11DB7`）。

---

## 6. ハンドルとプロトコル

UEFI の設計の中心。デバイスもイメージも「**ハンドル**」で表され、機能は「**プロトコル**」としてハンドルに付く。[§7.3]

```
ハンドルデータベース（ファームウェアが管理）

EFI_HANDLE (ディスク1) ─┬─ Block I/O Protocol       {GUID, interface*}
                        ├─ Disk I/O Protocol
                        └─ Device Path Protocol
EFI_HANDLE (ESP)       ─┬─ Simple File System Protocol
                        ├─ Block I/O Protocol
                        └─ Device Path Protocol
EFI_HANDLE (GPU)       ─┬─ Graphics Output Protocol
                        └─ Device Path Protocol
EFI_HANDLE (自分のイメージ) ┬ Loaded Image Protocol
                           └ Loaded Image Device Path Protocol
```

| 用語 | 意味 |
|---|---|
| **プロトコル** | GUID（名前）とインタフェース構造体（関数ポインタ＋インスタンスデータ）の組。C++ のインタフェース／Rust のトレイトオブジェクトに近い |
| **ハンドル** | 1 つ以上のプロトコルの集まり。不透明なポインタ |
| **ハンドルデータベース** | 「どのハンドルにどのプロトコルが付いているか」の表。ファームウェアが動的に管理 |
| **プロトコルインタフェース** | プロトコルの実体（構造体）へのポインタ。第 1 引数に `This` として自分自身を渡して関数を呼ぶ |

### プロトコルの探し方

| 関数 | 用途 |
|---|---|
| `LocateProtocol(GUID)` | そのプロトコルを持つ**最初の 1 つ**のインタフェースを直接取得。GOP のように「1 つあればよい」もの向け |
| `LocateHandleBuffer(ByProtocol, GUID)` | そのプロトコルを持つ**全ハンドル**の配列を取得（バッファは自動確保、`FreePool` で解放） |
| `LocateHandle(...)` | 同上だが呼び出し側がバッファを用意する |
| `LocateDevicePath(GUID, &DevicePath)` | デバイスパスに最も近いハンドルを探す |
| `OpenProtocol(Handle, GUID, ...)` | 特定のハンドルからインタフェースを取り出す（**推奨**） |
| `HandleProtocol(Handle, GUID)` | 同上の旧 API。仕様上「新しいコードは `OpenProtocol` を使うべき」とされている [§7.3.7] |
| `CloseProtocol(...)` | `OpenProtocol` で開いたものを閉じる |
| `ProtocolsPerHandle(Handle)` | そのハンドルに付いている全プロトコルの GUID 一覧 |

### `OpenProtocol` の Attributes [§7.3.9]

| 属性 | 値 | 主な利用者 | 意味 |
|---|---|---|---|
| `BY_HANDLE_PROTOCOL` | `0x01` | （`HandleProtocol` の内部実装） | 単に取得 |
| `GET_PROTOCOL` | `0x02` | ドライバ/アプリ | 取得のみ。アンインストールされても通知されない。Close 不要 |
| `TEST_PROTOCOL` | `0x04` | ドライバ | 存在確認のみ |
| `BY_CHILD_CONTROLLER` | `0x08` | バスドライバ | 子コントローラが使っていることを記録 |
| `BY_DRIVER` | `0x10` | ドライバ | ドライバとして使う。他ドライバの `BY_DRIVER` を排除 |
| `EXCLUSIVE` | `0x20` | アプリ | **排他的に使う**。`BY_DRIVER` で開いているドライバを `Stop()` させる |

アプリが `EXCLUSIVE` で開くと、そのプロトコルを使っていたドライバ（例: GOP の上に乗っているコンソールドライバ）が切り離される。アプリ終了時、アプリが開いたプロトコルは自動的に閉じられる。[§7.4.5]

### UEFI ドライバモデル（概要）

ドライバは自分自身のハンドルに **Driver Binding Protocol**（`Supported()` / `Start()` / `Stop()`）をインストールし、`ConnectController()` によってコントローラのハンドルに結び付けられる。アプリを書くだけなら「ファームウェアが起動時にほぼすべてのデバイスを接続済み」と考えてよい。[§7.3.1, §11]

---

## 7. Boot Services 一覧

`SystemTable->BootServices` から呼ぶ。`ExitBootServices()` 成功後は一切使えない。[§4.4, §7]

| 分類 | 関数 |
|---|---|
| TPL | `RaiseTPL`, `RestoreTPL` |
| メモリ | `AllocatePages`, `FreePages`, `GetMemoryMap`, `AllocatePool`, `FreePool` |
| イベント・タイマ | `CreateEvent`, `CreateEventEx`, `SetTimer`, `WaitForEvent`, `SignalEvent`, `CloseEvent`, `CheckEvent` |
| プロトコル操作 | `InstallProtocolInterface`, `ReinstallProtocolInterface`, `UninstallProtocolInterface`, `HandleProtocol`, `RegisterProtocolNotify`, `LocateHandle`, `LocateDevicePath`, `InstallConfigurationTable` |
| イメージ | `LoadImage`, `StartImage`, `Exit`, `UnloadImage`, **`ExitBootServices`** |
| その他 | `GetNextMonotonicCount`, `Stall`（μ秒単位のビジーウェイト）, `SetWatchdogTimer` |
| ドライバ接続 | `ConnectController`, `DisconnectController` |
| Open/Close | `OpenProtocol`, `CloseProtocol`, `OpenProtocolInformation` |
| ライブラリ的 | `ProtocolsPerHandle`, `LocateHandleBuffer`, `LocateProtocol`, `InstallMultipleProtocolInterfaces`, `UninstallMultipleProtocolInterfaces` |
| CRC | `CalculateCrc32` |
| その他 | `CopyMem`, `SetMem` |

### ウォッチドッグタイマ [§7.5.1]

ブートマネージャはブートオプションを起動する前に **5 分**のウォッチドッグを仕掛ける。5 分以内に `ExitBootServices()` しないと**システムがリセットされる**。

- `SetWatchdogTimer(0, 0, 0, NULL)` で無効化できる（Timeout = 0 で無効）。
- 対話的なアプリやデバッグで長く止まる場合は無効化しておく。
- `ExitBootServices()` 成功時には自動で無効になる。

---

## 8. TPL とイベント

### TPL (Task Priority Level) [§7.1]

UEFI には（ほぼ）スレッドがなく、代わりに**イベント通知関数**が優先度付きで割り込んでくる。その優先度が TPL。

| TPL | 値 | 用途 |
|---|---|---|
| `TPL_APPLICATION` | 4 | 通常の実行レベル。アプリはここで起動される。ユーザ I/O・ブロッキング可 |
| `TPL_CALLBACK` | 8 | 長めの処理（ファイルシステム、ディスク I/O）用の通知 |
| `TPL_NOTIFY` | 16 | 低レベル I/O 用。**ブロッキング禁止** |
| （ファームウェア割り込み） | — | ファームウェア内部専用 |
| `TPL_HIGH_LEVEL` | 31 | 割り込み禁止状態。ファームウェアが内部同期に使う |

- 高い TPL のコードは低い TPL のコードに割り込める。
- `RaiseTPL()` で一時的に上げると、それ以下の通知がマスクされる（クリティカルセクションのように使える）。`RestoreTPL()` で戻す。
- **多くのサービスに TPL 制限がある**。代表例:

| サービス | 呼べる TPL |
|---|---|
| `ExitBootServices()` | `= TPL_APPLICATION` |
| `WaitForEvent()` | `= TPL_APPLICATION` |
| `LoadImage()` | `< TPL_CALLBACK` |
| `Exit()` | `<= TPL_CALLBACK` |
| Simple File System / Block I/O / Disk I/O | `<= TPL_CALLBACK` |
| 変数サービス・時刻サービス | `<= TPL_CALLBACK` |
| メモリ確保・プロトコル操作 | `<= TPL_NOTIFY` |
| Simple Text Output | `<= TPL_NOTIFY` |
| `CreateEvent()` / `SetTimer()` | `< TPL_HIGH_LEVEL` |

（完全な表は §7.1 の "TPL Restrictions"）

### イベント [§7.1.1]

`CreateEvent(Type, NotifyTpl, NotifyFunction, Context, &Event)` で作る。

| Type | 値 | 意味 |
|---|---|---|
| `EVT_TIMER` | `0x80000000` | `SetTimer()` に渡せるタイマイベント。Boot Services 時間中のみ動作 |
| `EVT_RUNTIME` | `0x40000000` | ランタイムメモリから確保（`ExitBootServices()` 後にシグナルされるイベント用） |
| `EVT_NOTIFY_WAIT` | `0x00000100` | `WaitForEvent()`/`CheckEvent()` で待たれたときに通知関数が呼ばれる |
| `EVT_NOTIFY_SIGNAL` | `0x00000200` | シグナルされたときに通知関数が呼ばれる |
| `EVT_SIGNAL_EXIT_BOOT_SERVICES` | `0x00000201` | `ExitBootServices()` 時に通知 |
| `EVT_SIGNAL_VIRTUAL_ADDRESS_CHANGE` | `0x60000202` | `SetVirtualAddressMap()` 時に通知 |

- **イベントグループ**: `CreateEventEx()` で GUID を指定すると、同じグループのイベントがまとめてシグナルされる（例: `EFI_EVENT_GROUP_EXIT_BOOT_SERVICES`, `EFI_EVENT_GROUP_READY_TO_BOOT`）。
- **キー入力待ち**の定番: `WaitForEvent(1, &ConIn->WaitForKey, &Index)` → `ConIn->ReadKeyStroke()`。

---

## 9. メモリ管理

### 基本原則 [§7.2]

`ExitBootServices()` までは**メモリマップの所有者はファームウェア**。

1. メモリは必ず `AllocatePages` / `AllocatePool` で確保する。
2. 自分で確保したメモリ以外は使ってはいけない（「空いていそうだから」と勝手に書き込むのは違反）。
3. アプリは終了前に確保したものを解放する。

### 確保 API

| 関数 | 単位 | 備考 |
|---|---|---|
| `AllocatePages(Type, MemoryType, Pages, &Addr)` | 4 KiB ページ | 物理アドレスを指定できる |
| `AllocatePool(MemoryType, Size, &Buf)` | バイト | **8 バイトアライン**。中身はページから切り出される |

`AllocatePages` の `Type`:

| 値 | 意味 |
|---|---|
| `AllocateAnyPages` | どこでもよい（`*Addr` は無視） |
| `AllocateMaxAddress` | 末尾が `*Addr` 以下の範囲 |
| `AllocateAddress` | ちょうど `*Addr` から（カーネルを固定アドレスに置くときに使う。空いていなければ `EFI_NOT_FOUND`） |

- アプリの通常の確保は `EfiLoaderData` を使う。
- `MemoryType` の `0x70000000..0x7FFFFFFF` は OEM 用、**`0x80000000..0xFFFFFFFF` は OS ローダ用**に予約されている（独自タイプでカーネル領域に印を付けられる）。

### メモリタイプ (`EFI_MEMORY_TYPE`) [§7.2]

| 値 | 名前 | ExitBootServices 前 | ExitBootServices 後 |
|---|---|---|---|
| 0 | `EfiReservedMemoryType` | 使用不可 | 使用不可 |
| 1 | `EfiLoaderCode` | アプリのコード | **OS が自由に使える**（ただし呼び出したローダ自身が使用中） |
| 2 | `EfiLoaderData` | アプリのデータ・アプリの既定確保先 | **OS が自由に使える**（同上） |
| 3 | `EfiBootServicesCode` | BS ドライバのコード | **空き** |
| 4 | `EfiBootServicesData` | BS ドライバのデータ | **空き** |
| 5 | `EfiRuntimeServicesCode` | RT ドライバのコード | **保持必須** |
| 6 | `EfiRuntimeServicesData` | RT ドライバのデータ | **保持必須** |
| 7 | `EfiConventionalMemory` | 空き | **空き** |
| 8 | `EfiUnusableMemory` | エラーのあるメモリ | 使用不可 |
| 9 | `EfiACPIReclaimMemory` | ACPI テーブル | ACPI を読み終えるまで保持、その後は空き |
| 10 | `EfiACPIMemoryNVS` | ファームウェア予約 | 保持必須 |
| 11 | `EfiMemoryMappedIO` | ランタイムサービスが使う MMIO | OS は使わない（MMIO 情報は ACPI から得る） |
| 12 | `EfiMemoryMappedIOPortSpace` | I/O ポート変換用 MMIO | 同上 |
| 13 | `EfiPalCode` | プロセッサ用予約（Itanium） | 保持必須 |
| 14 | `EfiPersistentMemory` | 不揮発メモリ（NVDIMM など） | 同左 |
| 15 | `EfiUnacceptedMemoryType` | 未受け入れメモリ（TDX/SEV-SNP などの機密 VM） | 使う前に accept が必要 |

**OS が空きとして使えるもの** = `EfiLoaderCode`, `EfiLoaderData`, `EfiBootServicesCode`, `EfiBootServicesData`, `EfiConventionalMemory`（＋ACPI を読んだ後の `EfiACPIReclaimMemory`）。
ただし Loader 領域には**ローダ自身・カーネル・ブート情報**が載っているので、上書きしないよう注意。

### メモリマップ (`GetMemoryMap`) [§7.2.3]

```c
EFI_STATUS GetMemoryMap(
  IN OUT UINTN                 *MemoryMapSize,     // in: バッファサイズ / out: 必要サイズ
  OUT    EFI_MEMORY_DESCRIPTOR *MemoryMap,
  OUT    UINTN                 *MapKey,            // ExitBootServices に渡す
  OUT    UINTN                 *DescriptorSize,    // 1 要素のサイズ（重要）
  OUT    UINT32                *DescriptorVersion  // 現在 1
);

typedef struct {
  UINT32               Type;           // EFI_MEMORY_TYPE
  EFI_PHYSICAL_ADDRESS PhysicalStart;  // 4 KiB アライン
  EFI_VIRTUAL_ADDRESS  VirtualStart;   // SetVirtualAddressMap 用
  UINT64               NumberOfPages;  // 4 KiB ページ数（0 にはならない）
  UINT64               Attribute;      // 下表
} EFI_MEMORY_DESCRIPTOR;
```

- **`DescriptorSize` は `sizeof(EFI_MEMORY_DESCRIPTOR)` と一致するとは限らない**（将来の拡張のため）。配列を辿るときは必ず `DescriptorSize` 刻みで進める。
- バッファが小さいと `EFI_BUFFER_TOO_SMALL` と必要サイズが返る。そのサイズでバッファを確保すると**確保自体でマップが増える**ので、少し余裕を持たせる（仕様にも明記）。
- **`MapKey` はメモリマップが変わるたびに変わる**。
- メモリマップには「実在するメモリ」しか載らない。載っていない物理アドレス範囲を OS がどう使うかは OS の自由。
- PCI の MMIO（フレームバッファなど）は通常メモリマップに載らない（載るのはランタイムサービスが必要とする MMIO だけ）。

### メモリ属性 (`Attribute`) [§7.2.3]

**その領域が「設定可能な」能力**を表すビットであり、現在の設定値ではない点に注意。

| 名前 | 値 | 意味 |
|---|---|---|
| `EFI_MEMORY_UC` | `0x1` | キャッシュ不可にできる |
| `EFI_MEMORY_WC` | `0x2` | ライトコンバイニングにできる |
| `EFI_MEMORY_WT` | `0x4` | ライトスルーにできる |
| `EFI_MEMORY_WB` | `0x8` | ライトバックにできる（通常 RAM） |
| `EFI_MEMORY_UCE` | `0x10` | UC + fetch-and-add |
| `EFI_MEMORY_WP` | `0x1000` | ライトプロテクト（キャッシュ属性として） |
| `EFI_MEMORY_RP` | `0x2000` | 読み込み保護できる |
| `EFI_MEMORY_XP` | `0x4000` | 実行不可にできる |
| `EFI_MEMORY_NV` | `0x8000` | 不揮発メモリ |
| `EFI_MEMORY_MORE_RELIABLE` | `0x10000` | 他より信頼性が高い |
| `EFI_MEMORY_RO` | `0x20000` | 読み取り専用にできる |
| `EFI_MEMORY_SP` | `0x40000` | 特定用途メモリ（OS の汎用確保は避けるべき） |
| `EFI_MEMORY_CPU_CRYPTO` | `0x80000` | CPU のメモリ暗号化を適用可能 |
| `EFI_MEMORY_HOT_PLUGGABLE` | `0x100000` | ホットリムーブ可能 |
| `EFI_MEMORY_RUNTIME` | `0x8000000000000000` | **`SetVirtualAddressMap()` で仮想マッピングが必要**（ランタイム領域） |

---

## 10. ExitBootServices

OS ローダの山場。**ファームウェアからシステムの所有権を受け取る**操作。[§7.4.6]

```c
EFI_STATUS ExitBootServices(IN EFI_HANDLE ImageHandle, IN UINTN MapKey);
```

### 正しい手順

```
1. （必要なものを全部準備: カーネル読み込み、GOP 情報取得、ブート情報領域の確保 …）
2. GetMemoryMap() で最新のマップと MapKey を取得
3. 直後に ExitBootServices(ImageHandle, MapKey)
   ├─ EFI_SUCCESS            → 完了
   └─ EFI_INVALID_PARAMETER  → MapKey が古い。2 に戻って再取得・再呼び出し
```

- 2 と 3 の間に**メモリマップを変える操作をしない**こと。画面出力（`ConOut`）でもファームウェア内部で確保が起きうるので、間に `print` を挟まない。
- 1 回目の呼び出しが失敗した後、ファームウェアは**部分的にシャットダウン済みの可能性がある**。リトライまでの間に呼んでよいのは **メモリ確保系サービス（`GetMemoryMap` など）だけ**。
- `TPL_APPLICATION` で呼ぶこと。

### 成功後に起きること

- Boot Services と、すべてのハンドルベースのプロトコル（コンソール、GOP の `Blt`、ファイルシステム…）が**使用不可**になる。
- System Table の `ConIn`/`ConOut`/`StdErr` 系と `BootServices` が NULL になる。
- ウォッチドッグタイマが無効になり、タイマイベントも止まる。
- `EfiBootServicesCode/Data` を含む空きメモリはすべて**ローダのもの**になる。
- 引き続き使えるのは **Runtime Services** と、事前に控えておいた**情報**（メモリマップ、フレームバッファのアドレス、ACPI RSDP のアドレスなど）だけ。
- GOP のフレームバッファは**物理アドレスへの直接書き込み**ならそのまま使える（だから事前に `FrameBufferBase` などを保存しておく）。

---

## 11. Runtime Services と UEFI 変数

`SystemTable->RuntimeServices` から呼ぶ。`ExitBootServices()` 後も使える。[§4.5, §8]

| 分類 | 関数 |
|---|---|
| 時刻 | `GetTime`, `SetTime`, `GetWakeupTime`, `SetWakeupTime` |
| 仮想メモリ | `SetVirtualAddressMap`, `ConvertPointer` |
| 変数 | `GetVariable`, `GetNextVariableName`, `SetVariable`, `QueryVariableInfo` |
| その他 | `GetNextHighMonotonicCount`, **`ResetSystem`** |
| カプセル | `UpdateCapsule`, `QueryCapsuleCapabilities` |

### 再入の制限 [§8.1]

Runtime Services は**再入可能ではない**。ある呼び出しの実行中（割り込みされた場合や別 CPU で実行中の場合も含む）に、関連するサービスを呼んではいけない。例: 変数系の実行中に変数系を呼ぶ、時刻系の実行中に時刻系を呼ぶ、など。OS 側でロックをかけて直列化する。

### 実行時に未サポートのサービス

プラットフォームは「OS 稼働中は使えない Runtime Service」を `EFI_RT_PROPERTIES_TABLE`（Configuration Table）で通知できる。その場合でも関数は呼び出し可能で、`EFI_UNSUPPORTED` を返す。[§8.1, §4.6.2]

### `ResetSystem` [§8.5.1]

| ResetType | 意味 |
|---|---|
| `EfiResetCold` | コールドリセット（電源再投入相当） |
| `EfiResetWarm` | ウォームリセット（未対応ならコールド） |
| `EfiResetShutdown` | 電源断（ACPI S5 相当） |
| `EfiResetPlatformSpecific` | GUID 付きのプラットフォーム固有リセット |

カーネル初期段階で「とりあえず再起動・電源断したい」ときに便利。

### `SetVirtualAddressMap` [§8.4.1]

OS がランタイムサービスを高位仮想アドレスに移すための関数。

- `ExitBootServices()` 後に、**物理マッピングのまま**、**1 回だけ**呼べる（2 回目以降は `EFI_UNSUPPORTED`）。
- `EFI_MEMORY_RUNTIME` 属性を持つ全ディスクリプタに新しい仮想アドレスを与えたマップを渡す。
- 以後、ランタイムサービスは新しい仮想アドレスで呼ぶ必要がある。
- 呼ばなければ、ランタイム領域を仮想 = 物理でマップしておけば呼べる。自作 OS の初期段階では**呼ばない**のが簡単。

### UEFI 変数 [§8.2]

不揮発ストレージ（NVRAM）などに置かれる **名前（`CHAR16` 文字列）＋ベンダ GUID** で識別されるキー・バリュー。ブート設定、Secure Boot の鍵などがここにある。

```c
EFI_STATUS GetVariable(
  IN     CHAR16   *VariableName,
  IN     EFI_GUID *VendorGuid,
  OUT    UINT32   *Attributes OPTIONAL,
  IN OUT UINTN    *DataSize,
  OUT    VOID     *Data OPTIONAL
);
```

| 属性 | 値 | 意味 |
|---|---|---|
| `EFI_VARIABLE_NON_VOLATILE` (NV) | `0x01` | 再起動後も残る |
| `EFI_VARIABLE_BOOTSERVICE_ACCESS` (BS) | `0x02` | Boot Services 時間中にアクセス可 |
| `EFI_VARIABLE_RUNTIME_ACCESS` (RT) | `0x04` | ExitBootServices 後もアクセス可（BS も必須） |
| `EFI_VARIABLE_HARDWARE_ERROR_RECORD` | `0x08` | ハードウェアエラー記録用 |
| `EFI_VARIABLE_AUTHENTICATED_WRITE_ACCESS` | `0x10` | 非推奨 |
| `EFI_VARIABLE_TIME_BASED_AUTHENTICATED_WRITE_ACCESS` | `0x20` | 署名付き書き込みのみ許可（Secure Boot 鍵など） |
| `EFI_VARIABLE_APPEND_WRITE` | `0x40` | 追記 |

- **グローバル変数**（仕様で定義された変数）のベンダ GUID は `EFI_GLOBAL_VARIABLE` = `8BE4DF61-93CA-11D2-AA0D-00E098032B8C`。
- Linux からは `/sys/firmware/efi/efivars/<名前>-<GUID>` で見える（先頭 4 バイトが属性）。

---

## 12. ブートマネージャ

ファームウェア組み込みのブートマネージャが、UEFI 変数に従ってイメージを起動する。[§3]

### ブート関連のグローバル変数 [§3.3]

| 変数 | 型 | 意味 |
|---|---|---|
| `Boot####` | `EFI_LOAD_OPTION` | ブートオプション。`####` は**4 桁の 16 進数**（例: `Boot0001`） |
| `BootOrder` | `UINT16[]` | `Boot####` を試す順番 |
| `BootNext` | `UINT16` | **次回 1 回だけ**最優先で試す。試行前に削除される |
| `BootCurrent` | `UINT16` | 今回起動したオプション |
| `Timeout` | `UINT16` | ブートメニューの待ち秒数 |
| `Driver####` / `DriverOrder` | | 起動時にロードするドライバ |
| `SysPrep####` / `SysPrepOrder` | | `Boot####` の前に実行するアプリ |
| `ConIn` / `ConOut` / `ErrOut` | デバイスパス | 既定のコンソールデバイス |
| `PlatformLang` | ASCII | 言語（例: `en-US`） |
| `SecureBoot` | `UINT8` | 1 なら Secure Boot 有効 |
| `OsIndications` / `OsIndicationsSupported` | `UINT64` | OS → FW への要求（「次回 FW 設定画面を開く」など） |

### ロードオプション (`EFI_LOAD_OPTION`) [§3.1.3]

```c
typedef struct {
  UINT32  Attributes;           // LOAD_OPTION_ACTIVE (0x1) など
  UINT16  FilePathListLength;
  // CHAR16                   Description[];   NUL 終端の表示名
  // EFI_DEVICE_PATH_PROTOCOL FilePathList[];  起動するファイルのデバイスパス
  // UINT8                    OptionalData[];  イメージに渡すデータ（LoadOptions になる）
} EFI_LOAD_OPTION;
```

`efibootmgr`（Linux）はこれを編集するツール。

### リムーバブルメディアのフォールバックパス [§3.5.1.1]

デバイスパスにファイル名がない（USB メモリなど）場合、ファームウェアは既定のファイル名を補う。

| アーキテクチャ | パス |
|---|---|
| x64 | **`\EFI\BOOT\BOOTX64.EFI`** |
| IA-32 | `\EFI\BOOT\BOOTIA32.EFI` |
| AArch64 | `\EFI\BOOT\BOOTAA64.EFI` |
| RISC-V 64 | `\EFI\BOOT\BOOTRISCV64.EFI` |

`Boot####` が 1 つもない QEMU + OVMF でも、ESP にこのパスでファイルを置けば起動するのはこの仕組みによる。

---

## 13. 主要プロトコル

### Loaded Image Protocol [§9.1]

`ImageHandle` から開く。**自分自身についての情報**。

```c
typedef struct {
  UINT32                    Revision;
  EFI_HANDLE                ParentHandle;
  EFI_SYSTEM_TABLE         *SystemTable;
  EFI_HANDLE                DeviceHandle;     // ★自分が読み込まれたデバイス
  EFI_DEVICE_PATH_PROTOCOL *FilePath;         // そのデバイス上でのファイルパス
  VOID                     *Reserved;
  UINT32                    LoadOptionsSize;
  VOID                     *LoadOptions;      // コマンドライン相当
  VOID                     *ImageBase;        // ★ロードされたアドレス（デバッグに便利）
  UINT64                    ImageSize;
  EFI_MEMORY_TYPE           ImageCodeType;
  EFI_MEMORY_TYPE           ImageDataType;
  EFI_IMAGE_UNLOAD          Unload;
} EFI_LOADED_IMAGE_PROTOCOL;
```

「自分と同じパーティションにあるカーネルファイルを読む」には、`DeviceHandle` から Simple File System Protocol を開くのが定石。

### Device Path Protocol [§10]

デバイスの位置を表す**可変長ノードの連なり**。

```c
typedef struct {
  UINT8 Type;
  UINT8 SubType;
  UINT8 Length[2];   // このノードのバイト長（ヘッダ込み）。アライン不定なのでバイト配列
} EFI_DEVICE_PATH_PROTOCOL;  // この後にノード固有データが続く
```

| Type | 種類 | 例 |
|---|---|---|
| `0x01` | Hardware | `Pci(0x1F,0x2)` |
| `0x02` | ACPI | `PciRoot(0x0)` |
| `0x03` | Messaging | `Sata(...)`, `USB(...)`, `NVMe(...)`, `MAC(...)` |
| `0x04` | Media | `HD(1,GPT,<GUID>,...)`（パーティション）, `\EFI\BOOT\BOOTX64.EFI`（ファイルパス） |
| `0x05` | BIOS Boot Specification | レガシー |
| `0x7F` | End | SubType `0xFF` = パス全体の終端、`0x01` = インスタンスの区切り |

テキスト表記の例: `PciRoot(0x0)/Pci(0x1F,0x2)/Sata(0x0,0xFFFF,0x0)/HD(1,GPT,...)/\EFI\BOOT\BOOTX64.EFI`
変換には Device Path To Text / From Text Protocol を使う。[§10.6]

### Simple File System Protocol と File Protocol [§13.4, §13.5]

```
Simple File System Protocol
  └ OpenVolume() → ルートディレクトリの EFI_FILE_PROTOCOL
                     ├ Open(&New, L"\\kernel.elf", Mode, Attr)
                     ├ Read(&Size, Buf) / Write / Close / Delete
                     ├ GetPosition / SetPosition
                     ├ GetInfo(&EFI_FILE_INFO_ID, ...)   ← ファイルサイズ取得
                     └ Flush
```

- **パス区切りはバックスラッシュ `\`**。先頭が `\` ならボリュームのルートから、そうでなければ `This` からの相対パス。`.` と `..` も使える。
- 文字列は `CHAR16`。
- `OpenMode` の有効な組み合わせは **Read**、**Read|Write**、**Create|Read|Write** の 3 つだけ。

| 定数 | 値 |
|---|---|
| `EFI_FILE_MODE_READ` | `0x1` |
| `EFI_FILE_MODE_WRITE` | `0x2` |
| `EFI_FILE_MODE_CREATE` | `0x8000000000000000` |

- ファイルサイズは `GetInfo()` で `EFI_FILE_INFO`（`FileSize`, `PhysicalSize`, 作成/更新時刻, `Attribute`, `FileName[]`）を取って得る。`EFI_FILE_INFO` は**可変長**（末尾がファイル名）なので、バッファが足りないと `EFI_BUFFER_TOO_SMALL` と必要サイズが返る。
- ディレクトリに対する `Read()` は `EFI_FILE_INFO` を 1 エントリずつ返す（サイズ 0 で終端）。

### Simple Text Input / Output Protocol [§12.3, §12.4]

`SystemTable->ConIn` / `ConOut` / `StdErr`。

- `ConOut->OutputString(This, L"Hello\r\n")`: **改行は `\r\n`**（LF はカーソルを下げるだけ、CR は行頭へ戻すだけ）。
- `ConOut->ClearScreen()`, `SetAttribute()`（色）, `SetCursorPosition()`, `QueryMode()`/`SetMode()`（テキストモードの行数・列数）
- `ConIn->ReadKeyStroke(This, &Key)`: キーが無ければ `EFI_NOT_READY`。待つなら `WaitForEvent(ConIn->WaitForKey)`。
- `EFI_INPUT_KEY` は `ScanCode`（矢印・ファンクションキーなど）と `UnicodeChar` の組。
- 修飾キーやホットキーが必要なら Simple Text Input **Ex** Protocol。

### Graphics Output Protocol (GOP) [§12.9]

画面描画用。BIOS 時代の VGA/VBE の代わり。`LocateProtocol(&gEfiGraphicsOutputProtocolGuid)` で取るのが一般的。

```c
struct EFI_GRAPHICS_OUTPUT_PROTOCOL {
  QueryMode(This, ModeNumber, &SizeOfInfo, &Info);  // モード情報を取得
  SetMode(This, ModeNumber);                        // モード変更（画面はクリアされる）
  Blt(...);                                         // 矩形転送（BootServices 中のみ）
  EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE *Mode;
};

typedef struct {
  UINT32                                 MaxMode;          // モード数（0..MaxMode-1）
  UINT32                                 Mode;             // 現在のモード番号
  EFI_GRAPHICS_OUTPUT_MODE_INFORMATION  *Info;
  UINTN                                  SizeOfInfo;
  EFI_PHYSICAL_ADDRESS                   FrameBufferBase;  // ★
  UINTN                                  FrameBufferSize;  // ★
} EFI_GRAPHICS_OUTPUT_PROTOCOL_MODE;

typedef struct {
  UINT32                     Version;
  UINT32                     HorizontalResolution;  // 幅（ピクセル）
  UINT32                     VerticalResolution;    // 高さ
  EFI_GRAPHICS_PIXEL_FORMAT  PixelFormat;           // ★
  EFI_PIXEL_BITMASK          PixelInformation;      // PixelBitMask のときだけ有効
  UINT32                     PixelsPerScanLine;     // ★
} EFI_GRAPHICS_OUTPUT_MODE_INFORMATION;
```

| `PixelFormat` | 1 ピクセル（32 bit）のバイト並び |
|---|---|
| `PixelRedGreenBlueReserved8BitPerColor` | byte0 = R, byte1 = G, byte2 = B, byte3 = 予約 |
| `PixelBlueGreenRedReserved8BitPerColor` | byte0 = B, byte1 = G, byte2 = R, byte3 = 予約（QEMU/OVMF は通常こちら） |
| `PixelBitMask` | `PixelInformation` のマスクで定義 |
| `PixelBltOnly` | **リニアフレームバッファなし**（`Blt()` でしか描けない → ExitBootServices 後は描画不可） |

- **ピクセル (x, y) のアドレス = `FrameBufferBase + 4 * (PixelsPerScanLine * y + x)`**。`HorizontalResolution` ではなく `PixelsPerScanLine` を使う（行末にパディングがありうる）。[§12.9.2]
- 原点 (0, 0) は左上。
- `ExitBootServices()` 後も、フレームバッファへの直接書き込みは有効。カーネルに渡すべき情報は `FrameBufferBase`, `FrameBufferSize`, 解像度, `PixelsPerScanLine`, `PixelFormat`。

### その他よく使うプロトコル

| プロトコル | 用途 | 節 |
|---|---|---|
| Block I/O | ブロックデバイスの生読み書き（LBA 単位） | §13.9 |
| Disk I/O | バイト単位のディスク読み書き | §13.7 |
| Load File / Load File 2 | ネットワークブート等でファイルを取得 | §13.1, §13.2 |
| Serial I/O | シリアルポート | §12.8 |
| PCI Root Bridge I/O / PCI I/O | PCI コンフィグ空間・BAR アクセス | §14 |
| RNG | 乱数（KASLR などに） | §37.5 |
| Device Path Utilities / To Text | デバイスパス操作・文字列化 | §10.5, §10.6 |
| Memory Attribute | ページの R/W/X 属性の変更（新しめ） | §37.7 |

---

## 14. ディスク: GPT と ESP

### GPT (GUID Partition Table) [§5.3]

```
LBA 0          Protective MBR（タイプ 0xEE のパーティション 1 つでディスク全体を覆う）
LBA 1          GPT ヘッダ（シグネチャ "EFI PART"）
LBA 2..        パーティションエントリ配列
...            パーティション本体
末尾付近       バックアップのエントリ配列
最終 LBA       バックアップ GPT ヘッダ
```

- 各パーティションは**パーティションタイプ GUID** と**一意な GUID** を持つ。
- ヘッダとエントリ配列は CRC32 で保護される。

### ESP (EFI System Partition) [§13.3]

- パーティションタイプ GUID: **`C12A7328-F81F-11D2-BA4B-00A0C93EC93B`**
- ファイルシステムは **FAT**（ESP は FAT32 が基本。リムーバブルメディアは FAT12/16/32 も可）。ファームウェアは FAT12/16/32 すべてをサポートしなければならない。
- ルートに `EFI` ディレクトリを置き、その下にベンダごとのディレクトリ（`\EFI\<vendor>\`）を作る。
- リムーバブルメディアでは `\EFI\BOOT\BOOT<arch>.EFI`（§12）。

QEMU では `-drive format=raw,file=fat:rw:<ディレクトリ>` で、ホストのディレクトリを FAT ディスクとして見せられる（QEMU の機能。UEFI 仕様ではない）。

---

## 15. Configuration Table

System Table の `ConfigurationTable[NumberOfTableEntries]` は **{GUID, ポインタ}** の配列で、ファームウェアが OS に渡すテーブル群の置き場所。アドレスは物理アドレスで、ランタイムへの移行時にも書き換えられない。[§4.6]

```c
typedef struct {
  EFI_GUID  VendorGuid;
  VOID     *VendorTable;
} EFI_CONFIGURATION_TABLE;
```

| テーブル | GUID | 指す先 |
|---|---|---|
| **ACPI 2.0 以降** (`EFI_ACPI_20_TABLE_GUID`) | `8868E871-E4F1-11D3-BC22-0080C73C8881` | **RSDP**（ACPI の入口。OS で必須） |
| ACPI 1.0 (`ACPI_TABLE_GUID`) | `EB9D2D30-2D88-11D3-9A16-0090273FC14D` | RSDP (ACPI 1.0) |
| SMBIOS 3.x (`SMBIOS3_TABLE_GUID`) | `F2FD1544-9794-4A2C-992E-E5BBCF20E394` | SMBIOS 3.0 エントリポイント |
| SMBIOS (`SMBIOS_TABLE_GUID`) | `EB9D2D31-2D88-11D3-9A16-0090273FC14D` | SMBIOS 2.x エントリポイント |
| Devicetree (`EFI_DTB_TABLE_GUID`) | `B1B621D5-F19C-41A5-830B-D9152C69AAE0` | FDT（主に ARM） |
| RT Properties (`EFI_RT_PROPERTIES_TABLE_GUID`) | `EB66918A-7EEF-402A-842E-931D21C38AE9` | 実行時に使える Runtime Service のビットマスク |
| Memory Attributes (`EFI_MEMORY_ATTRIBUTES_TABLE_GUID`) | `DCFA911D-26EB-469F-A220-38B7DC461220` | ランタイム領域の詳細な R/X 属性 |

ACPI 2.0 の GUID があればそちらを優先する（XSDT が使える）。

---

## 16. Secure Boot の概要

ファームウェアが**署名を検証したイメージだけを起動する**仕組み。[§32]

| 鍵 / DB | 変数名 | 役割 |
|---|---|---|
| **PK** (Platform Key) | `PK` | プラットフォーム所有者の鍵。1 つだけ。KEK の更新を許可する |
| **KEK** (Key Exchange Key) | `KEK` | OS ベンダ等の鍵。db/dbx の更新を許可する |
| **db** | `db` | 起動を**許可**する署名・証明書・ハッシュ |
| **dbx** | `dbx` | 起動を**禁止**する署名・ハッシュ（失効リスト） |

- PK・KEK は `EFI_GLOBAL_VARIABLE` の GUID、db・dbx は `EFI_IMAGE_SECURITY_DATABASE_GUID`（`D719B2CB-3D3A-4596-A3BC-DAD00E67656F`）の下にある。
- 署名は PE イメージ内の Authenticode 署名。
- モード:
  - **Setup Mode**（`SetupMode` = 1）: PK 未登録。鍵を認証なしで書き換えられる。
  - **User Mode**: PK 登録済み。鍵の更新には署名が必要。
  - **Audit Mode** / **Deployed Mode**: 監査用 / 最も厳格なモード。
- 自作 OS 開発中は Secure Boot を気にしなくてよい。PK が登録されていない Setup Mode では署名検証は強制されないため、空の `OVMF_VARS` から始めた QEMU では署名なしの `.efi` がそのまま起動する。

---

## 17. ステータスコード

`EFI_STATUS` は `UINTN`。**最上位ビットが 1 ならエラー**、0 なら成功または警告。[付録 D]

x64 では エラーコード = `0x8000000000000000 | n`。

| 種別 | 名前 | n | よくある原因 |
|---|---|---|---|
| 成功 | `EFI_SUCCESS` | 0 | — |
| エラー | `EFI_LOAD_ERROR` | 1 | イメージのロード失敗 |
| | `EFI_INVALID_PARAMETER` | 2 | 引数不正。`ExitBootServices` では **MapKey が古い** |
| | `EFI_UNSUPPORTED` | 3 | 未対応の操作 |
| | `EFI_BAD_BUFFER_SIZE` | 4 | バッファサイズが要求に合わない |
| | `EFI_BUFFER_TOO_SMALL` | 5 | **バッファ不足。必要サイズが返る**（再確保して再呼び出し） |
| | `EFI_NOT_READY` | 6 | データ未到着（キー入力なし等） |
| | `EFI_DEVICE_ERROR` | 7 | ハードウェアエラー |
| | `EFI_WRITE_PROTECTED` | 8 | 書き込み禁止 |
| | `EFI_OUT_OF_RESOURCES` | 9 | メモリ等の不足 |
| | `EFI_VOLUME_CORRUPTED` | 10 | ファイルシステム破損 |
| | `EFI_VOLUME_FULL` | 11 | 空き容量なし |
| | `EFI_NO_MEDIA` | 12 | メディアなし |
| | `EFI_MEDIA_CHANGED` | 13 | メディアが交換された |
| | `EFI_NOT_FOUND` | 14 | 見つからない（ファイル、プロトコル、`AllocateAddress` の範囲が使用中など） |
| | `EFI_ACCESS_DENIED` | 15 | アクセス拒否（排他オープン中など） |
| | `EFI_NO_RESPONSE` | 16 | 応答なし |
| | `EFI_NO_MAPPING` | 17 | マッピングなし |
| | `EFI_TIMEOUT` | 18 | タイムアウト |
| | `EFI_NOT_STARTED` | 19 | 未開始 |
| | `EFI_ALREADY_STARTED` | 20 | 開始済み |
| | `EFI_ABORTED` | 21 | 中断 |
| | `EFI_INCOMPATIBLE_VERSION` | 25 | バージョン不一致 |
| | `EFI_SECURITY_VIOLATION` | 26 | セキュリティ違反（Secure Boot で署名不正など） |
| | `EFI_CRC_ERROR` | 27 | CRC エラー |
| | `EFI_END_OF_MEDIA` | 28 | メディア終端 |
| | `EFI_END_OF_FILE` | 31 | ファイル終端 |
| 警告 | `EFI_WARN_UNKNOWN_GLYPH` | 1 | 表示できない文字があった（`OutputString`） |
| | `EFI_WARN_DELETE_FAILURE` | 2 | 閉じたが削除できなかった |
| | `EFI_WARN_WRITE_FAILURE` | 3 | 閉じたがフラッシュできなかった |
| | `EFI_WARN_BUFFER_TOO_SMALL` | 4 | データが切り詰められた |

`EFI_BUFFER_TOO_SMALL` で「必要サイズを教えてもらって再呼び出し」は UEFI 全体で頻出するパターン（`GetMemoryMap`, `GetVariable`, `File->GetInfo`, `LocateHandle` など）。

---

## 18. 落とし穴チェックリスト

- [ ] **文字列は UCS-2**。`CHAR16` の NUL 終端。改行は `\r\n`。
- [ ] **呼び出し規約は Microsoft x64**（`efiapi`）。関数ポインタ型を自作するときに System V にしない。
- [ ] **`DescriptorSize` 刻みでメモリマップを辿る**。`sizeof` を使わない。
- [ ] **`GetMemoryMap` 用バッファは余裕を持って確保**。確保でマップが伸びる。
- [ ] **`GetMemoryMap` → `ExitBootServices` の間に何もしない**（print もしない）。失敗したらマップを取り直してリトライ。
- [ ] **ExitBootServices 後に Boot Services・プロトコル・`ConOut` を使わない**。必要な情報は事前に退避。
- [ ] **フレームバッファの行幅は `PixelsPerScanLine`**。`PixelFormat` を確認する（RGB/BGR、`PixelBltOnly`）。
- [ ] **ウォッチドッグは 5 分**。長時間止まるなら `SetWatchdogTimer(0, ...)`。
- [ ] **Boot Services 中は確保したメモリだけを使う**。カーネルを固定アドレスに置くなら `AllocateAddress` で予約する。
- [ ] **Boot Services 中に CR3/GDT/IDT を差し替えない**（差し替えるなら ExitBootServices 後）。
- [ ] **ランタイム領域（`EfiRuntimeServices*`, `EfiACPIMemoryNVS`）は OS でも保持する**。上書きすると Runtime Services や ACPI が壊れる。
- [ ] **`ExitBootServices` 後の空きメモリには Loader 領域も含まれる**が、そこにはローダ自身・カーネル・ブート情報があるので、OS のアロケータに渡す前に区別する。
- [ ] **`EFI_STATUS` は最上位ビットでエラー判定**。警告（非 0 で最上位ビット 0）を失敗扱いしない。
- [ ] **GUID のメモリ上の並び**は先頭 3 フィールドがリトルエンディアン。
- [ ] **ファイルパスは `\` 区切り**。

---

## 19. 用語集

| 用語 | 説明 |
|---|---|
| **UEFI** | Unified Extensible Firmware Interface。ファームウェアと OS 間のインタフェース仕様。UEFI Forum が策定 |
| **EFI** | UEFI の前身（Intel の EFI 1.10）。型名や関数名の `EFI_` 接頭辞はその名残 |
| **PI** | Platform Initialization 仕様。ファームウェア内部の構造を定める |
| **EDK II / TianoCore** | UEFI/PI のオープンソース参照実装 |
| **OVMF** | Open Virtual Machine Firmware。QEMU/KVM 用の EDK II ビルド |
| **CSM** | Compatibility Support Module。UEFI 上でレガシー BIOS を模倣する仕組み（近年は廃止傾向） |
| **UEFI イメージ** | PE32+ 形式の UEFI 実行ファイル（`.efi`） |
| **UEFI アプリケーション** | Subsystem 10 のイメージ。終了するとアンロードされる |
| **UEFI ドライバ** | Subsystem 11/12 のイメージ。成功すると常駐し、プロトコルを提供する |
| **UEFI OS ローダ** | `ExitBootServices()` を呼んでシステムを引き継ぐ UEFI アプリケーション |
| **ImageHandle** | 実行中のイメージ自身を表すハンドル。エントリポイントの第 1 引数 |
| **System Table** | 全サービス・コンソール・Configuration Table への入口。エントリポイントの第 2 引数 |
| **Boot Services** | `ExitBootServices()` までだけ使えるサービス群（メモリ、イベント、プロトコル、イメージ） |
| **Runtime Services** | OS 稼働中も使えるサービス群（時刻、変数、リセット、仮想アドレス変換） |
| **ExitBootServices** | Boot Services を終了させ、システムの制御を OS ローダへ移す関数 |
| **ハンドル (EFI_HANDLE)** | プロトコルの集まりを表す不透明ポインタ。デバイスやイメージを表す |
| **プロトコル** | GUID で識別される関数テーブル＋データ。UEFI の機能はほぼすべてこの形 |
| **GUID** | 128 ビットの一意識別子。プロトコル・変数・テーブル・パーティションタイプの識別に使う |
| **ハンドルデータベース** | ハンドルとプロトコルの対応表。ファームウェアが管理 |
| **Driver Binding Protocol** | UEFI ドライバモデルでドライバがコントローラに接続するためのプロトコル |
| **コントローラ** | ドライバが管理する対象のハンドル（デバイス） |
| **デバイスパス** | デバイスやファイルの位置を表すノードの列（`PciRoot(0)/Pci(1,0)/...`） |
| **TPL** | Task Priority Level。イベント通知の優先度・割り込みマスクの単位 |
| **イベント** | 非同期通知の仕組み。タイマ、キー入力待ち、ExitBootServices 通知などに使う |
| **イベントグループ** | GUID で束ねたイベント群。まとめてシグナルされる |
| **メモリマップ** | 物理メモリの使用状況を表す `EFI_MEMORY_DESCRIPTOR` の配列 |
| **MapKey** | メモリマップのバージョン番号。マップが変わると変わる |
| **メモリタイプ** | 領域の用途（`EfiConventionalMemory` など）。ExitBootServices 前後で扱いが変わる |
| **Pool / Pages** | 確保の単位。Pool はバイト単位（8 バイトアライン）、Pages は 4 KiB 単位 |
| **アイデンティティマップ** | 仮想アドレス = 物理アドレスとなるページング設定。Boot Services 中はこれ |
| **SetVirtualAddressMap** | ランタイムサービスを OS の仮想アドレス空間へ移す関数（1 回限り） |
| **UEFI 変数** | 名前＋GUID で識別される（主に不揮発の）キー・バリュー |
| **NVRAM** | UEFI 変数を保存する不揮発領域（通常は SPI フラッシュ） |
| **ブートマネージャ** | 変数に従ってブートオプションを起動するファームウェア内の機能 |
| **Boot#### / BootOrder** | ブートオプションとその順序を表す変数 |
| **ロードオプション** | `Boot####` の中身。表示名＋デバイスパス＋オプションデータ |
| **ESP** | EFI System Partition。FAT でフォーマットされた、UEFI イメージを置くパーティション |
| **GPT** | GUID Partition Table。UEFI 仕様が定めるパーティション方式 |
| **Protective MBR** | GPT ディスクの LBA 0 に置く、MBR しか知らないツールからディスクを守るためのダミー MBR |
| **GOP** | Graphics Output Protocol。画面モード設定とフレームバッファ情報の取得 |
| **フレームバッファ** | 画面の各ピクセルに対応するメモリ領域。直接書けば画面に出る |
| **Blt** | Block Transfer。GOP の矩形転送関数（Boot Services 中のみ） |
| **ConIn / ConOut / StdErr** | System Table が持つ既定のテキスト入出力 |
| **Configuration Table** | GUID とポインタの組の配列。ACPI・SMBIOS などの場所を OS に伝える |
| **RSDP** | Root System Description Pointer。ACPI テーブル群の入口 |
| **SMBIOS** | ハードウェア情報（メーカー、メモリスロット等）のテーブル |
| **Secure Boot** | 署名を検証したイメージのみ起動する仕組み |
| **PK / KEK / db / dbx** | Secure Boot の鍵階層と許可・禁止データベース |
| **ウォッチドッグタイマ** | ブートオプション起動時に 5 分でセットされるタイマ。時間切れでリセット |
| **BSP / AP** | Bootstrap Processor（起動時に動いている CPU）/ Application Processor（それ以外の CPU）。UEFI アプリは BSP で動く |
| **UEFI Shell** | UEFI 上のコマンドラインシェル（`Shell.efi`）。`map`, `ls`, `memmap`, `dh` などでデバッグに便利 |
| **カプセル** | ファームウェア更新などのデータを OS からファームウェアに渡す仕組み（`UpdateCapsule`） |

---

## 20. 参考資料

### 一次資料

- **UEFI Specification 2.11**（HTML 版）: <https://uefi.org/specs/UEFI/2.11/>
  - 主な章: §2 Overview（実行環境・呼び出し規約）、§3 Boot Manager、§4 System Table、§7 Boot Services、§8 Runtime Services、§9 Loaded Image、§10 Device Path、§12 Console（GOP 含む）、§13 Media Access（ファイルシステム、ESP）、§5 GPT、§32 Secure Boot、付録 D Status Codes
- **UEFI 仕様一覧**（PI、ACPI、Shell 仕様も含む）: <https://uefi.org/specifications>
- **PI Specification**: <https://uefi.org/specifications>（ブートフェーズ SEC/PEI/DXE/BDS の定義）

### 仕様に準じる資料

- **EDK II `MdePkg/Include`**: <https://github.com/tianocore/edk2/tree/master/MdePkg/Include>
  - `Uefi/UefiSpec.h`（System Table、Boot/Runtime Services）、`Uefi/UefiMultiPhase.h`（メモリタイプ、変数属性）、`Uefi/UefiBaseType.h`（基本型、ステータスコード）、`Protocol/*.h`（各プロトコル）、`Guid/*.h`（各 GUID）
  - 定数値や構造体の並びを確認するのに便利（仕様書よりも grep しやすい）
- **uefi-rs**（Rust バインディング）: <https://docs.rs/uefi>, <https://docs.rs/uefi-raw>（`uefi-raw` は仕様の型をほぼそのまま Rust にしたもの）

### 本プロジェクトでの関連資料

- 『ゼロからのOS自作入門』1〜4 章: UEFI ローダの実装例（EDK II ベース）
- `bootloader` クレート 0.11: Rust による UEFI ローダの実装例
