# sais-rs

Implementation of the SAIS algorithm in safe Rust.

## Features
- Generic algorithm for any index and character type
- Safe
- No sentinel value needed (changes needed were taken from [suffix](https://github.com/BurntSushi/suffix))
- "Fast": about as fast as [this old benchmark](https://sites.google.com/site/yuta256/sais) on my local machine (absolut times).
  Probably a lot slower than the comparison since my machine is relatively fast.

## Benchmarks
| Name          | Time [s]  | Comparison Time [s] |
|---------------|-----------|---------------------|
| abac          | **0.004** | 0.010               |
| abba          | 1.2432    | **1.024**           |
| book1x20      | 3.4762    | **2.756**           |
| fib_s14930352 | 3.2305    | **1.828**           |
| fss9          | **0.104** | 0.218               |
| fss10         | 2.3122    | **1.446**           |
| houston       | **0.086** | 0.138               |
| paper5x80     | **0.036** | 0.054               |
| test1         | **0.088** | 0.118               |
| test2         | **0.089** | 0.122               |
| test3         | **0.082** | 0.114               |
