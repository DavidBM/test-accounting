# Test Accounting

This project showcases a transaction processor which takes its input from a CSV and outputs another CSV with the resulting accounts state.

## TL;DR

Run `cargo test` to execute a varied set of tests. You can check `tests/tests.rs` to see several usages of the library version of the crate. You can use `cargo run -- transactions.csv > accounts.csv` to run the executable. I advise to use the `--release` flat to test the executable.

## Design Decisions

I've tried several designs. Mostly related with parallelization. You can see the different approaches in the commits that start with "poc" (ex: "poc 4: rayon". In which I tried rayon in order to parallelize transaction processing).

### Parallelization

I tried several designs in order to parallelize the process. The main problem with CSV is that its parsing cannot be parallelized, so the ideal situation for parsing a CSV is having a thread only parsing the CSV and sending work to other thread/s.

The operations in itself aren't complex, and it is hard that they are going to suppose a bigger processing cost than the CSV row parsing itself, as they are usually simple additions and subtractions.

During the development I tried several ways to try to parallelize the work, but they all showed to be slower than just having 2 threads, one reading/parsing and other processing transfers.

As a summary for fast comparative when processing a CSV with 21 million entries (without white-spaces handling):
- 1 thread parsing and processing: 6.5 seconds.
- 1 thread parsing + rayon (with Dashmap) for processing: 12 seconds
- 1 thread parsing + thread pool (with data sharding) for processing: 9 seconds
- 1 thread parsing + 1 thread processing: 4.8 seconds

The 1+1 design works best compared to others 1+>1 due to these reasons:
- *With data sharding*: Multi thread solutions require to reorder the input. This is due to the dependency between messages. Ex: You can only resolve a dispute if the dispute is already processed. So each thread keeps a shard/chunk of the accounts (like Kafka multi consumers) and handles all operations belonging to their accounts. 
- *With Dashmap or similar*: Having more than 1 thread incurs in synchronization primitives as `Arc`s and, in the worse case (rayon), `Mutex`es. Which in this case, given that the operation in itself is so small, hurts more than benefits.
- *In general*: 1+1 allows each thread to have a tighter loop that only do one thing, which I suspect that helps the CPU branch predictor to be more efficient.

### Parallelization Correctness

For keeping the correctness of the system, the most important thing when thinking on the concurrency part is that **message processing must be serializable (as in serializable isolation level) per account**. Messages from different account can be processed in parallel.

Initially I thought to just use rayon to parallelize the processing, but this point made impossible to "just use rayon TM" because it requires some ordering before processing.

### Parallelization Design and Data Structures Chosen

Finally, I chose the *1 thread parsing + 1 thread processing* solution as it performs better and it isn't so much more complicated compared to the *1 thread parsing and processing*.

For the thread communication I use `crossbeam-channel` with a 25 MB buffer (in my experiments, it was the best performant size).

For the account *"storage"* in ram, I use a `BTreeMap<u16, Account>` as it performed better than a HashMap. I didn't change the hasher when using the HashMap because choosing a hasher is something that needs to be done depending on the execution context and the default Rust hasher is a safe bet for all cases.

For the deserialization, I implemented a custom deserialization of account to match the desired output. It is very simple, and it can be seen in `<crate::account::Account as Serialize>`.

For the numeric handling, I'm using `decimal-rs` in order to avoid IEEE-754 floating-point calculation errors. As a thing to consider, if the program is going to handle astronomically absurd huge quantities (like, as many dollars as atoms are in the galaxy or something like it) then I would use `bigdecimal` or keep `decimal-rs` and use `.checked_add`/`.checked_sub`/etc method family rather than `+` and `-` operators.

### Error Handling

In this project there are two types of errors mainly. Errors that can be ignored, and errors that need to abort the execution. For the later I use `eyre`, so I just bubble up with `?` all errors.

There is usually a third family of errors, the ones that can be handled. Mostly in servers and long-running programs. For the current problem and execution context (shell binary) I didn't find any errors of that family.

In the case this executable needs to be extended to handle errors (ex: report error on stderr) we will need to use `thiserror` to be able to differentiate the errors to be handled and the ones that cannot be handled and need to abort the program. Mostly in a shape like:

```rust
#[derive(thiserror::Error)]
pub enum MyError{
    #[error("Error context. {0}")]
    MyBusinessError(OtherErrorType),
    //... other error types ...
    #[error("Error context. {0}")]
    Unexpected(#[from] eyre::Report),
}
```

## Code Design

I've separated the application in a `main.rs` and a `lib.rs`. The `lib.rs` provides an interface as 
```rust
fn process_csv<R: Read, W: Write>(reader: R, writer: &mut W) -> Result<()>
```
which doesn't care of how input and output comes, as long as it can be `.read()` and `.write()`. 

The main.rs file makes sure to create and wrap the `File` in a `BufRead` and the output in a `BufWrite` for performance.

The testing becomes much simpler when having a generic interface in the `lib.rs` as you can provide simple string to test the code. Example:

```rust
#[test]
fn chargeback_dispute() {
    let input = r#"type,client,tx,amount
deposit,14,1,57097.49
dispute,14,2,16397.12
chargeback,14,2,
deposit,14,1,57097.49
"#;
    let expected_output = r#"client,available,held,locked,total
14,40700.37,16397.12,true,57097.49
"#;

    let mut output: Vec<u8> = vec![];

    process_csv(input.as_bytes(), &mut output).unwrap();

    assert_eq!(expected_output, &String::from_utf8(output).unwrap())
}
```

Beyond that, all the *"business logic"* is encapsulated in the `account.rs`. And finally, the `processor.rs` encapsulates the processing strategy (AKA, multi-thread, +1 thread, same thread, etc). This is the main file that changed when trying different parallelization strategies.

## Business Logic Implementation Decisions

For the implementation, I took these decisions:
- Locked accounts don't process anything. This includes other `dispute` and `resolve` transactions. It is reasonable to require other disputes to be processed in order to have the most updated available/held funds in the output. That would be a very easy change in the `crate::account::Account::dispute` and `crate::account::Account::resolve` methods.
- Each account tracks its opened disputes. When processing a resolve transaction, the opened dispute is removed. In the case of a chargeback, the account is locked and the dispute is not removed. This works well with the previous point. If the previous point was required to change as exposed, the `chargeback` transaction will remove the dispute from the account. Again, easy change in the `crate::account::Account::chargeback` method.

## Things I Didn't Dry

I didn't try to use any executor as `async-std` or `tokio` as this code has no network dependencies. The IO is with the filesystem and can easily be solved with `BufRead` and `BufWrite` and no async solution will be able to outperform that in the current context (executable binary). 

Now, this is a binary to be executed in a local shell and that reads a CSV in the local filesystem. In the case of using this code in a HTTP server or to have required saving the accounts in a database like QLDB or PostgreSQL, I would have used an executor. Provably async-std, as it is simpler to use for small utilities. That would have changed the whole game. 

In such case, the transaction serialization per account is still required, but that can be easily solved with `dashmap` or similar and/or a task-pool (not thread pool).

Also, I had the idea of trying to just parse the csv row as string and then try to deserialize it in parallel. But that has the problem that then they need to be reordered after the parallel deserialization in order to not have `resolve` transactions before the `dispute` transaction. I suspect that that reordering will make the parallelization non-effective.

## Performance Numbers

Best case for 21 Million (597 MB file) transactions read from the CSV, processed and saved to a CSV: 12.5 seconds in my i7-9750H with SSD.

There is an important thing to be noted. When disabling the `.trim(csv::Trim::All)` on the `csv` library, the processing time goes down to **4.8 seconds**. I left it with the trim enabled, as that is a hard requirement. If performance is paramount, I would create a custom parser using `nom` or similar crate for the specific CSV format used in production. I've done parsers in the past (https://github.com/Couragium/ion-binary-rs and https://github.com/Couragium/qldb-rs) and it is an accessible thing to do safely in Rust.  

## Dependencies

- clap: executable arguments parsing
- crossbeam-channel: channel between main thread and worker thread 
- csv: csv parsing / encoding
- decimal: decimal handling
- eyre: error handling library (like anyhow, but a bit more ergonomic)
- serde: generic serialization/deserialization

`cargo audit` is clean.

`cargo geiger` shows that the library `csv` and `decimal-rs` directly use unsafe. I don't like that. I trust `eyre`, `clap` and `crossbeam-channel`, but I would question is unsafe is really needed in a csv parser and a decimal handling library.

`cargo clippy` is clean

`cargo test` is clean

`cargo build` is clean
