# Bip39 miner
Trying to increase throughput as much as possible in a cpu ethereum address tester

### Architecture
We have 3 types of threads:
- UI
    - handles visualization of application vitals:
        - tries/s
        - false positive rate
        - last checked addresses
- Worker
    - generates random number and checks in the xorfilter
    - if true, sends to checker thread
- Checker
    - accumulates addresses that got sent for double-checking in the last second
    - checks them all at once, saves the ones that are actually present

Only one thread is necessary for the UI and the Checker each.
Worker threads should be as many as possible, i.e., N-2 where N is the number of cores.

