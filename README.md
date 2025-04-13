# Ethereum PK Miner
Trying to increase throughput as much as possible in a cpu ethereum PK miner

### Architecture
We have 3 types of threads:
- UI
    - handles visualization of application throughputs, from tries to specific functions.
- Worker
    - generates random number and checks in the xorfilter
    - if true, sends to checker thread
- Checker
    - checks addresses that got sent for double-checking with a sqlite db containing all ethereum addresses

Only one thread is necessary for the UI and the Checker each.

Worker threads should be as many as possible, i.e., N-2 where N is the number of cores,
though in my 28 core system doing only 8 worker threads leads to only around 20% less throughput but way less cpu usage.

### Steps to run
#### Bigquery public ethereum data
First, we need to get all existing accounts with some ether balance above 0.
For that, we can fetch google's public ethereum data using bigquery.

Pre-requisites:
- a google cloud account
- python3 + pandas + google cloud library
- google-cli to authenticate the current session

then, run the script:
```bash
python3 data/query.py
```
It will take some time to fetch all the data.

#### Preparing the xorfilter
The xorfilter will be used for the 1st check of address existence, to build it run
```bash
cargo run --release prepare

Options:
  -c, --csv-path <CSV_PATH>    Solution file to expand [default: ./data/accounts.csv]
  -f, --fuse <FUSE>            Which binary fuse to use, must be a value of 8, 16, 32 [default: 16]
      --fuse-path <FUSE_PATH>  Where to save the fuse, defaults to `./data/xorfilter{fuse}` [default: ]
  -h, --help                   Print help
```

#### Preparing the sqlite db
Pre-requisites:
- sqlite cli

For making the csv file into a useful sqlite db, we can import it directly using sqlite commands:
```bash
sqlite3 data/data.db
.mode csv
.import data/accounts.csv accounts
create index accounts_address on accounts(address)
```

#### Running
```bash
cargo run --release miner

Options:
  -t, --threads <THREADS>      How many worker threads should be spawned, if empty will use the num_cpus crate [default: 0]
  -f, --fuse <FUSE>            Which binary fuse to use, must be a value of 8, 16, 32 [default: 16]
      --fuse-path <FUSE_PATH>  Where the fuse is saved, if empty will read `./data/xorfilter{fuse}` [default: ]
  -h, --help                   Print help
```

### Results
On my laptop's i7-14700HX, running on 26 worker threads:
![image](https://github.com/user-attachments/assets/7d87144f-e377-4afe-9b51-b11441fe9364)

On 8 worker threads:
![image](https://github.com/user-attachments/assets/b47a037e-3026-42b1-a927-2536be6ed303)
