# `phantom` CLIツール 開発メモ

## 🎯 ツール概要

`phantom` は、Git Worktree を便利に扱うための CLI ツール。  
複数の作業ツリー（= gardens）を管理し、それぞれにコーディングエージェント（= phantom）を「召喚」するという世界観をベースに設計する。

---

## 🔧 技術選定

- 使用言語: **Go**
  - CLIツール開発に適しており、ビルドして単一バイナリにできる
- CLIライブラリ（予定）: `cobra`

---

## 🧱 基本構成

### メインコマンド

```
phantom
```

### サブコマンドと機能

| サブコマンド | 概要 |
|--------------|------|
| `garden create <name>` | 新しい worktree（garden）を作成 |
| `garden list` | 作成済みの garden 一覧を表示 |
| `garden switch <name>` | 指定した garden に切り替え |
| `garden delete <name>` | garden を削除 |
| `spawn <garden-name> <command>` | garden 上で任意のコマンドを実行（phantom を召喚）|
| `kill <garden-name> <command>` | garden 上で動作中のプロセスを終了（phantom を消滅）|
| `list` | 動作中の phantom（プロセス）一覧を表示 |

---

## 🧠 補足事項・世界観

- `garden`: git worktree の各作業ディレクトリを指す。
- `phantom`: コーディングエージェントやエディタなど、各作業ツリー上で動かすプロセスの比喩的表現。
- `spawn`, `kill`: 実行中プロセスの管理をわかりやすく表現した言葉。
- `phantom list`: 起動中の phantom 一覧を表示（例: `docker ps` 相当）。

---

## 📦 プロセス管理（予定）

- `PM2` の Node.js API に触発され、Goでもプロセスの起動・終了・一覧機能を実装予定。
- 起動中プロセスは JSON などで永続化し、状態管理に使う（例: `~/.phantom/processes.json`）。

