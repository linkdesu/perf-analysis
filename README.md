# perf-analysis

构建合约可以执行以下命令:

``` sh
capsule build
```

测试用例放在 `tests/src/tests.rs` 运行的话执行下面命令即可看到运行时开销：

``` sh
cargo test -p tests -- --nocapture
```

布隆过滤器本身的代码位于 `libs/das-bloom-filter/src/bloom_filter.rs` ，问题就出在这里面的 `BloomFilter.new_with_data` 方法运行时使用 cycles 数过多。
