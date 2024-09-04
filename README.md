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
* [x] Read scored moves from database (not exhaustively tested).
* [x] Load pvs.
* [ ] Bench multi-pv feasibility.
* [ ] Correctly handle mate scores.
* [ ] Fallback key for variant positions.
* [ ] Data model for user provided analysis.
* [ ] Server implementation and protocol discussion.
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

lila-ws API
-----------

Input:

```json
{
   "t":"evalGet",
   "d":{
      "fen":"rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 3",
      "path":"",
      "variant":"fromPosition",
      "mpv":2
   }
}
```

Output:

```json
{
   "t":"evalHit",
   "d":{
      "fen":"rnbqkbnr/pppp1ppp/8/4p3/6P1/5P2/PPPPP2P/RNBQKBNR b KQkq - 0 3",
      "knodes":133793,
      "depth":30,
      "pvs":[
         {
            "moves":"d8h4",
            "mate":-1
         },
         {
            "moves":"h7h5 g4g5 d8g5 f1h3 g5h4 e1f1 b8c6 b1c3 g8e7 d2d3",
            "cp":-248
         }
      ],
      "path":""
   }
}
```


Acknowledgements
----------------

Thanks [@noobpwnftw](https://github.com/noobpwnftw) for https://chessdb.cn and sharing the database dumps.

Thanks for [@vondele](https://github.com/vondele) for providing a [reference implementation of the chessdb.cn binary format](https://github.com/vondele/cdbdirect).
