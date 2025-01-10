# OS! (基于 Blog OS 的模块实验创新)

## 一、队伍简介 (选题及成员等)
| 项目     | 内容                   |
| -------- | ---------------------- |
| 所属学校 | 合肥工业大学宣城校区   |
| 比赛方向 | 模块实验创新           |
| 队伍编号 | T202419359994461       |
| 队伍名   | OS!                    |
| 队伍成员 | 王卫东、董嘉轩、胡子龙 |
| 指导老师 | 田卫东、周红鹃         |

## 二、项目完成情况介绍
### 项目分别位于以下 3 个分支:
- [Ex1](https://gitlab.eduxiji.net/T202419359994461/project2608132-275359/-/tree/ex1)
- [Ex2](https://gitlab.eduxiji.net/T202419359994461/project2608132-275359/-/tree/ex2)
- [Ex3](https://gitlab.eduxiji.net/T202419359994461/project2608132-275359/-/tree/ex3)

## Building

This project requires a nightly version of Rust because it uses some unstable features. At least nightly _2020-07-15_ is required for building. You might need to run `rustup update nightly --force` to update to the latest nightly even if some components such as `rustfmt` are missing it.

## Running

You can run the disk image in [QEMU] through:

[QEMU]: https://www.qemu.org/

```
cargo run
```

[QEMU] and the [`bootimage`] tool need to be installed for this.


## License

- MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

### Contribution
