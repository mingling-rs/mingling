<h1 align="center">Mingling 中文文档 🇨🇳</h1>

<p align="center">
    该页面为 <b>Mingling</b> 命令行框架的中文文档，由 <a href="https://github.com/Weicao-CatilGrass">Weicao-CatilGrass</a> 维护
</p>

## 前言

如果您希望编写出架需要长期维护，子命令很多的单体命令行程序，
并且同时需要能够上下文感知的命令行补全体验，那 Mingling 一定值得您学习！

## 关于 Mingling

Mingling 的 Slogan 是：“Macro magician in your CLI.（您命令行里的宏魔法师）”，
它通过宏系统，构建了一套清晰的描述命令行程序的范式。

Mingling 永久免费使用，您可以选择 [MIT](https://github.com/mingling-rs/mingling/blob/main/LICENSE-MIT) 或 [APACHE 2.0](https://github.com/mingling-rs/mingling/blob/main/LICENSE-APACHE) 中的任意一个协议使用它。

Mingling 的设计目标是：

- 类型和状态驱动：类型描述状态，函数处理行为，宏标记它们！
- 编译期侧重：编译时完全烧录所有命令列表、状态、类型，保持运行时轻量。
- 副作用隔离：将行为处理的副作用通过框架机制尽可能隔离，创造干净的上下文。
- 可测试性：所有步骤都是函数，可轻松断言和注入上下文。
- 轻松组合生态：能力可通过 Rust 生态接入，框架提供纯粹的调度逻辑。

## 使用前，您必须知道的一些事：

### 1. 单 Crate 架构

您可以使用外部的框架无关逻辑拆分 Crate，
同时 Mingling 也提供了 Setup、Resource、Hook 等可拆分的拓展点。

但是，最终命令行的所有 **绑定层**（即命令和实际行为之间的关联层）被严格限制在同一个 Crate 中，
如果您的业务需要拆分该绑定层，请酌情考虑。

### 2. 关于稳定性

Mingling 目前仍在积极开发中，且 API 的变化较为频繁，如果您需要开发的程序是 **生产级** 的，
非常不推荐使用 Mingling。

### 3. Parser VS Framework

Mingling 并非一款 CLI Parser，而是 CLI Framework，它并不内置参数解析的能力。

但是 Mingling 提供了两个特性可以便您接入 [`clap`](https://github.com/clap-rs/clap) 或者更适合 Mingling 的 [`arg-picker`](https://github.com/catilgrass/arg-picker) 外部 Parser

---

## 开始

当然，如果您使用 Mingling 的动机是 "Just for fun"，那这是最好的状态了！
