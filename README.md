# swap-pool

Do you need to store a large amount of data and it may not fit into your computer's RAM? Then this library is created for you!

## Usage

```rust
use swap_pool::prelude::*;

// Create a new swap pool with 128 bytes of memory available
// and designate a "swap" folder for it
let mut pool = SwapPool::new(128, "swap");

// Spawn 3 entities in the pool
// Each entity must implement From and Into Vec<u8> traits
let a = pool.spawn(vec![0; 128]).unwrap();
let b = pool.spawn(vec![1; 128]).unwrap();
let c = pool.spawn(vec![2; 128]).unwrap();

// Check if spawned entities are hot (stored in the RAM)
// "b" and "c" will be saved to the "swap" folder because
// "a" will take all 128 available bytes (a bit more actually)
dbg!(a.is_hot()); // a.is_hot() = true
dbg!(b.is_hot()); // b.is_hot() = false
dbg!(c.is_hot()); // c.is_hot() = false

// Flush all the entities to the disk ("a" will become cold)
pool.handle().flush().unwrap();

// Read entities values. Note that this operation
// will always clone the value so use x2 amount of RAM
// due to some inner magic
// (I need to share ownership of the value if there's no memory available)
assert!(a.value().unwrap() == vec![0; 128]);
assert!(b.value().unwrap() == vec![1; 128]);
assert!(c.value().unwrap() == vec![2; 128]);

// Check entities status
// Since we can keep only one entity hot at a time
// the pool will free an old one and replace it by a new one
// so firstly we allocated "a", then it was replaced by "b",
// and finally it was replaced by "c"
dbg!(a.is_hot()); // a.is_hot() = false
dbg!(b.is_hot()); // b.is_hot() = false
dbg!(c.is_hot()); // c.is_hot() = true

// Update the value stored in entity "c"
// Second update will return "false" because we can't
// allocate at least 1024 bytes in the current swap pool
// (maximum 128 bytes available)
dbg!(c.update(vec![0; 64]).unwrap());   // c.update(vec![0 ; 64]).unwrap() = true
dbg!(c.update(vec![0; 1024]).unwrap()); // c.update(vec![0 ; 1024]).unwrap() = false

// Show some statistics about the memory use
// Note: "used" will report 0 because second "update"
// has flushed the entity and didn't update its value
// because it didn't have enough free space available
println!("Total: {}", pool.handle().allocated());
println!(" Used: {}", pool.handle().used());
println!(" Free: {}", pool.handle().available());
```

Notes:

1. You can use `entity.value_allocate()` to ignore pool memory limitations and always make the entity hot. Call this method if you want to keep the entity in the RAM as long as possible.
2. On the contrary, `entity.value_unallocate()` will return stored value (or read it from the disk) and flush the entity, making it cold. Call this method if you don't need to access the entity often.
3. You can replace the entity's value using `entity.replace(value)`. It will not try to free the memory to store the new value.
4. You can free any amount of memory you need by calling `handle.free(memory)`. It will also say if it succeeded to free given amount of memory.
5. You can also call `handle.flush()` to flush all the entities.
6. Call `handle.collect_garbage()` to remove weak references to the dropped entities. Otherwise they will stack up in the pool's entities list.
7. You can create swap pools with different entities ranking implementations using `SwapPool::with_manager()`. By default `SwapLastUseManager` is used, which records timestamps of when records' values were requested. You can use `SwapUpgradeCountManager` to count their amount instead, or make your own implementation of the `SwapManager` trait.

## Features

| Name | Description |
| - | - |
| `thiserror` | Implement `Error` trait for `SwapError` type from the [thiserror](https://crates.io/crates/thiserror) crate. |
| `timestamp-uuid` [1] | Use `SystemTime::now()` to generate entities' UUIDs. Enabled by default. |
| `random-uuid` [1] | Use [rand](https://crates.io/crates/rand) crate to generate random entities' UUIDs. |
| `crc32-uuid` [2] | Use [crc32fast](https://crates.io/crates/crc32fast) crate to generate random entities' UUIDs. |
| `xxhash-uuid` [2] | Use [xxhash-rust](https://crates.io/crates/xxhash-rust) crate (xxh3) to generate random entities' UUIDs. |
| `size-of-crate` [3] | Implement `SizeOf` trait for all the types supported by [size-of](https://crates.io/crates/size-of) crate. |
| `dyn-size-of-crate` [3] | Implement `SizeOf` trait for all the types supported by [dyn_size_of](https://crates.io/crates/dyn_size_of) crate. |
| `full` | `thiserror`, `random-uuid`, `xxhash-uuid`, `dyn-size-of-crate` |
| `default` | `timestamp-uuid` |

Notes:

1. Enabling `random-uuid` will disable `timestamp-uuid` and use of values to generate UUID (result will be based on the randomly generated number only). When both disabled - UUID generation will be based on the entity's value only.
2. If both `crc32-uuid` and `xxhash-uuid` enabled - the latest one will be prioritized. If none - default `HashMap`'s hasher is used.
3. You can't enable both `size-of-crate` and `dyn-size-of-crate` features simultaneously because it would cause compatibility issues. Consider enabling only one of them.

## Entities keep alive ranking

> Sorry github users, but crates.io has monospaced font.

Let's say we have 3 entities in the swap pool:

```
 Entities
┌────────────────────────────────────────┐
│ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│ │ Entity 1 │ │ Entity 2 │ │ Entity 3 │ │
│ └──────────┘ └──────────┘ └──────────┘ │
└────────────────────────────────────────┘
```

And we've requested "Entity 2" and "Entity 3"'s values. Then our keep alive ranks array will look like this:

```
 Keep alive ranks
┌───────────────────────────┐
│ ┌──────────┐ ┌──────────┐ │
│ │ Entity 2 │ │ Entity 3 │ │
│ └──────────┘ └──────────┘ │
└───────────────────────────┘
```

Now, if we need to free some memory, swap pool will go through the entities list according to their rank and flush them. Firstly it will try to flush all the entities without a rank (which weren't requested for some time) - "Entity 1" in our case. Then swap pool will go through the entities which have the rank in ascending order - so try to flush "Entity 2" first, and "Entity 3" later. If there's no more entities remaining - return "false" status.

Author: [Nikita Podvirnyi](https://github.com/krypt0nn)\
Licensed under [MIT](LICENSE)
