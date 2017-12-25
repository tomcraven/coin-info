# coin-info

A small experiment into a rust crypto currency ticker. Run using cargo and pass
a list of symbols to track:
```
cargo run btc eth ltc
```

Output looks something like this:
```
┌───────────────────────────────────────────────────────────────────────────────┐
│index  symbol     usd        1h         24h        7d         24h volume       │
│                                                                               │
│1      BTC        14193.9    0.67       8.01       -24.79     10579900000.0    │
│2      ETH        766.836    0.27       17.7       0.62       2457490000.0     │
│3      LTC        275.515    0.1        5.66       -17.17     853805000.0      │
└───────────────────────────────────────────────────────────────────────────────┘
```
