lila-cloudeval
==============

Experimental cloud eval server for https://lichess.org. Work in progress.

Current idea
------------

In the vast majority of cases, the https://chessdb.cn will provide much better analysis than our current cloud evals.
We probably still want user provided evals (live broadcasts, shared studies), but only accept them if chessdb.cn does not already have something better.
*Better* to be determined based on heuristics: Satisfy requested number of pvs, depth, pv length.

* [x] Basic terarkdb bindings, capable of loading the database dump.
* [x] Create normalized database key for chess positions.
* [ ] Fallback key for variant positions.
* [x] Read scored moves from database (not exhaustively tested).
* [ ] Load pvs.
* [ ] Data model for user provided analysis.
* [ ] Server implementation.
* [ ] Integrate into `lila`.
* [ ] Integrate into `lila-ws`.

Usage
-----

Run a test roughly comparable to `cdbdirect_threaded` from https://github.com/vondele/cdbdirect:

```sh
git submodule update --init
(cd terarkdb-sys/terarkdb && ./build.sh)
cargo run --release --bin cdbdirect -- --db-path /mnt/ssd/chess-20240814/data caissa_sorted_100000.epd
```

Acknowledgements
----------------

Thanks [@noobpwnftw](https://github.com/noobpwnftw) for https://chessdb.cn and sharing the database dumps.

Thanks for [@vondele](https://github.com/vondele) for providing a [reference implementation of the chessdb.cn binary format](https://github.com/vondele/cdbdirect).
