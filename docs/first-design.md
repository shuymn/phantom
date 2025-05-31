# git phantom

`git phantom` は `git worktree` をより便利に扱える CLI ツールです。

## 🎯 目的

Gitの `worktree` 機能に以下の便利操作を提供する：

- ワークツリーの作成
- ワークツリーの一覧表示
- ワークツリーの切り替え（ディレクトリ移動）
- ワークツリーの削除

## 💡 コンセプト

「phantom（幻影）」という名前は、`git worktree` で作成される “分身” のような存在にちなんでおり、縦横無尽に動ける・実体はあるが本体とは分離しているという特徴を表現しています。

## ⚙️ 実装方針

- 使用言語：**Go 言語**
- クロスプラットフォーム対応（Mac / Linux / Windows）
- 将来的な配布を想定し、**単一バイナリとして提供可能にする**

## 🧪 提供予定のサブコマンド

| サブコマンド例 | 機能 | 補足 |
|----------------|------|------|
| `git phantom list` | ワークツリーの一覧表示 | `git worktree list` のラッパー |
| `git phantom add <path> [-b <branch>]` | 新しいワークツリーの作成 | ブランチ指定対応予定 |
| `git phantom switch <path>` | 指定ワークツリーに移動（cd 補助） | `eval $(git phantom switch ...)` 形式を想定 |
| `git phantom remove <path>` | ワークツリーの削除 | `git worktree remove` のラッパー |
| `git phantom prune` | 存在しないワークツリーの掃除 | オプションで自動実行も可能にする想定 |

## 📝 備考

- CLIラッパー名は `git-phantom` とし、Git の外部サブコマンドとして動作する
- `$PATH` に `git-phantom` バイナリが存在すれば `git phantom` として呼び出される

---

## 🧭 今後の展望（アイデアメモ）

- `fzf` との連携による対話的切り替え
- よく使うワークツリーへのエイリアス機能
- 現在の作業ツリーからの自動補完
