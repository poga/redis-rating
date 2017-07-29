# redis-rating

A Redis module that helps you calculate the real rating from positive/negative rating feedback.

The algorithm used is described at [how not to sort by average rating](http://www.evanmiller.org/how-not-to-sort-by-average-rating.html).

## Install

Build from source:

```
$ git clone https://github.com/poga/redis-rating.git
$ cd redis-rating
$ cargo build --release
$ cp target/release/libredis_rating.dylib /path/to/modules/
```

Run Redis pointing to the newly built module:

```
redis-server --loadmodule /path/to/modules/libredis_rating.so
```

Alternatively add the following to a `redis.conf` file:

```
loadmodule /path/to/modules/libredis_rating.so
```

## Usage

### RT.RATEPOS

```
RT.RATEPOS <key> [<count>]
```

Add `count` positive rating to `key`. `count` is default to 1.

#### Response

The command will respond with an array of integers. The first integer is the total number of positive votes. The second integer is the total number of votes.

### RT.RATENEG

```
RT.RATEPOS <key> [<count>]
```

Add `count` negative rating to `key`. `count` is default to 1.

#### Response

The command will respond with an array of integers. The first integer is the total number of positive votes. The second integer is the total number of votes.

### RT.GET

```
RT.RATEPOS <key>
```

Estimate the **Real** score of the given key. `Score = Use Lower bound of Wilson score confidence interval for a Bernoulli parameter`

#### Response

Returns a double, which is the estimated score of key.

## License

This module is based on brandur's [redis-cell](https://github.com/brandur/redis-cell).

The following files is copied from brandur's project. Therefore preserve the original [license](https://github.com/brandur/redis-cell/blob/master/LICENSE)

```
The MIT License, Copyright (c) 2016 Brandur and contributors
* `src/redis/*`
* `src/error.rs`
* `src/marcos.rs`
```

Everything else is licensed under The MIT License.
