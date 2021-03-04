use super::*;
use ckb_testtool::context::Context;
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use das_bloom_filter::BloomFilter;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

// const MAX_CYCLES: u64 = 10_000_000_000;
const MAX_CYCLES: u64 = u64::MAX;

fn optimal_bits_count(capacity: f64, err_rate: f64) -> f64 {
    let ln_2_2 = std::f64::consts::LN_2.powf(2f64);

    // m = -1 * (n * ln ε) / (ln 2)^2
    (-1f64 * capacity * err_rate.ln() / ln_2_2).ceil()
}

//#[cfg(feature = "std")]
fn optimal_hashers_count(err_rate: f64) -> f64 {
    // k = -log_2 ε
    (-1f64 * err_rate.log2()).ceil()
}

#[test]
fn test_success() {
    // deploy contract
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary("perf-analysis");
    let out_point = context.deploy_cell(contract_bin);

    // prepare scripts
    let lock_script = context
        .build_script(&out_point, Bytes::from(vec![42]))
        .expect("script");
    let lock_script_dep = CellDep::new_builder().out_point(out_point).build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script)
            .build(),
    ];

    let outputs_data = vec![Bytes::new(); 2];

    // 上面都是无关紧要的构造交易的代码可以忽略

    // 创建布隆过滤器
    let rate = 0.0001;
    let sum = 5_000f64;
    let bits_count = optimal_bits_count(sum, rate);
    let hash_fn_count = optimal_hashers_count(rate);
    let mut all_items = Vec::new();
    let mut bf = BloomFilter::new(bits_count as u64, hash_fn_count as u64);

    // 插入 80% 的随机数据
    let s = (sum * 0.8) as u64;
    for _ in 1..s {
        let item: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(8)
            .collect();
        all_items.push(item.to_owned());
        bf.insert(item.as_bytes());
    }
    // 插入一个固定数据
    bf.insert(b"das");

    // 拼接一些用于回复布隆过滤器的参数后，放到交易的 witnesses 中
    let b_u32 = bits_count as u32;
    let h_u32 = hash_fn_count as u32;
    let mut v_32 = b_u32.to_le_bytes().to_vec();
    let h_32 = h_u32.to_le_bytes().to_vec();
    v_32.extend(h_32);
    let filter = bf.export_bit_u8();
    v_32.extend(filter);

    let witnesses = vec![Bytes::from(v_32)].pack();

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .witnesses(witnesses)
        .build();
    let tx = context.complete_tx(tx);

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);
}
