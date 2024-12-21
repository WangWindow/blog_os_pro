# OS! (基于 Blog OS 的模块实验创新)

| 项目       | 内容               |
|-----------|--------------------|
| 所属学校   | 合肥工业大学宣城校区   |
| 比赛方向   | 模块实验创新         |
| 队伍编号   | T202419359994461   |
| 队伍名     | OS!                |
| 队伍成员   | 王卫东、董嘉轩、胡子龙 |
| 指导老师   | 田卫东、周红鹃     |

## 对项目完成情况做一个介绍


## PDF文档和其他过程性材料链接
[材料1]()

## 代码的参考情况
[https://github.com/phil-opp/blog_os/tree/post-12](https://github.com/phil-opp/blog_os/tree/post-12)

## Building

This project requires a nightly version of Rust because it uses some unstable features. At least nightly _2020-07-15_ is required for building. You might need to run `rustup update nightly --force` to update to the latest nightly even if some components such as `rustfmt` are missing it.

You can build the project by running:

```
cargo build
```

To create a bootable disk image from the compiled kernel, you need to install the [`bootimage`] tool:

[`bootimage`]: https://github.com/rust-osdev/bootimage

```
cargo install bootimage
```

After installing, you can create the bootable disk image by running:

```
cargo bootimage
```

This creates a bootable disk image in the `target/x86_64-blog_os/debug` directory.

Please file an issue if you have any problems.

## Running

You can run the disk image in [QEMU] through:

[QEMU]: https://www.qemu.org/

```
cargo run
```

[QEMU] and the [`bootimage`] tool need to be installed for this.

## Testing

To run the unit and integration tests, execute `cargo xtest`.

## License

- MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)

### Contribution
