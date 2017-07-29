#[macro_use]
extern crate bitflags;
extern crate libc;
extern crate time;

#[macro_use]
mod macros;

pub mod error;
mod redis;

use error::CellError;
use libc::c_int;
use redis::Command;
use redis::raw;

const MODULE_NAME: &'static str = "redis-rating";
const MODULE_VERSION: c_int = 1;

struct RatePositiveCommand {
}

impl Command for RatePositiveCommand {
    fn name(&self) -> &'static str {
        "rt.ratepos"
    }

    // Run the command.
    fn run(&self, r: redis::Redis, args: &[&str]) -> Result<(), CellError> {
        if args.len() != 2 && args.len() != 3 {
            return Err(error!("Usage: {} <key> [<count>]", self.name()));
        }

        // the first argument is command name "cl.throttle" (ignore it)
        let key = args[1];
        let count = match args.get(2) {
            Some(n) => parse_i64(n)?,
            None => 1,
        };

        let positive_key = r.open_key_writable(&format!("rating:{}:positive", key));
        let positive_vote = match positive_key.read()? {
            Some(n) => if n.is_empty() { 0 } else { parse_i64(&n)? },
            None => 0,
        };
        let new_positive_vote = positive_vote + count;
        positive_key.write(&format!("{}", new_positive_vote))?;

        let total_key = r.open_key_writable(&format!("rating:{}:total", key));
        let total_vote = match total_key.read()? {
            Some(n) => if n.is_empty() { 0 } else { parse_i64(&n)? },
            None => 0,
        };
        let new_total_vote = total_vote + count;
        total_key.write(&format!("{}", new_total_vote))?;

        r.reply_array(2)?;
        r.reply_integer(new_positive_vote)?;
        r.reply_integer(new_total_vote)?;

        Ok(())
    }

    // Should return any flags to be registered with the name as a string
    // separated list. See the Redis module API documentation for a complete
    // list of the ones that are available.
    fn str_flags(&self) -> &'static str {
        "write"
    }
}

struct RateNegativeCommand {
}

impl Command for RateNegativeCommand {
    fn name(&self) -> &'static str {
        "rt.rateneg"
    }

    // Run the command.
    fn run(&self, r: redis::Redis, args: &[&str]) -> Result<(), CellError> {
        if args.len() != 2 && args.len() != 3 {
            return Err(error!("Usage: {} <key> [<count>]", self.name()));
        }

        // the first argument is command name "cl.throttle" (ignore it)
        let key = args[1];
        let count = match args.get(2) {
            Some(n) => parse_i64(n)?,
            None => 1,
        };

        let positive_key = r.open_key_writable(&format!("rating:{}:positive", key));
        let positive_vote = match positive_key.read()? {
            Some(n) => if n.is_empty() { 0 } else { parse_i64(&n)? },
            None => 0,
        };

        let total_key = r.open_key_writable(&format!("rating:{}:total", key));
        let total_vote = match total_key.read()? {
            Some(n) => if n.is_empty() { 0 } else { parse_i64(&n)? },
            None => 0,
        };
        let new_total_vote = total_vote + count;
        total_key.write(&format!("{}", new_total_vote))?;

        r.reply_array(2)?;
        r.reply_integer(positive_vote)?;
        r.reply_integer(new_total_vote)?;

        Ok(())
    }

    // Should return any flags to be registered with the name as a string
    // separated list. See the Redis module API documentation for a complete
    // list of the ones that are available.
    fn str_flags(&self) -> &'static str {
        "write"
    }
}

struct CalculateRatingCommand {
}

impl Command for CalculateRatingCommand {
    fn name(&self) -> &'static str {
        "rt.get"
    }

    // Run the command.
    fn run(&self, r: redis::Redis, args: &[&str]) -> Result<(), CellError> {
        if args.len() != 2 {
            return Err(error!("Usage: {} <key>", self.name()));
        }

        // the first argument is command name "cl.throttle" (ignore it)
        let key = args[1];

        let positive_key = r.open_key_writable(&format!("rating:{}:positive", key));
        let positive_vote = match positive_key.read()? {
            Some(n) => if n.is_empty() { 0 } else { parse_i64(&n)? },
            None => 0,
        };

        let total_key = r.open_key_writable(&format!("rating:{}:total", key));
        let total_vote = match total_key.read()? {
            Some(n) => if n.is_empty() { 0 } else { parse_i64(&n)? },
            None => 0,
        };

        let result;
        if total_vote == 0 {
            result = 0.0;
        } else {
            let phat = 1.0 * positive_vote as f64 / total_vote as f64;
            let z = 1.96; // 0.95 confidence level
            let n = total_vote as f64;
            result = (phat + z * z / (2.0 * n) -
                      z * (phat * (1.0 - phat) + z * z / (4.0 * n)) / n)
                .sqrt() / (1.0 + z * z / n);
        }

        r.reply_double(result)?;

        Ok(())
    }

    // Should return any flags to be registered with the name as a string
    // separated list. See the Redis module API documentation for a complete
    // list of the ones that are available.
    fn str_flags(&self) -> &'static str {
        "write"
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RatePositive_RedisCommand(ctx: *mut raw::RedisModuleCtx,
                                            argv: *mut *mut raw::RedisModuleString,
                                            argc: c_int)
                                            -> raw::Status {
    Command::harness(&RatePositiveCommand {}, ctx, argv, argc)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RateNegative_RedisCommand(ctx: *mut raw::RedisModuleCtx,
                                            argv: *mut *mut raw::RedisModuleString,
                                            argc: c_int)
                                            -> raw::Status {
    Command::harness(&RateNegativeCommand {}, ctx, argv, argc)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn CalculateRating_RedisCommand(ctx: *mut raw::RedisModuleCtx,
                                               argv: *mut *mut raw::RedisModuleString,
                                               argc: c_int)
                                               -> raw::Status {
    Command::harness(&CalculateRatingCommand {}, ctx, argv, argc)
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn RedisModule_OnLoad(ctx: *mut raw::RedisModuleCtx,
                                     argv: *mut *mut raw::RedisModuleString,
                                     argc: c_int)
                                     -> raw::Status {
    if raw::init(ctx,
                 format!("{}\0", MODULE_NAME).as_ptr(),
                 MODULE_VERSION,
                 raw::REDISMODULE_APIVER_1) == raw::Status::Err {
        return raw::Status::Err;
    }

    let ratePositiveCommand = RatePositiveCommand {};
    if raw::create_command(ctx,
                           format!("{}\0", ratePositiveCommand.name()).as_ptr(),
                           Some(RatePositive_RedisCommand),
                           format!("{}\0", ratePositiveCommand.str_flags()).as_ptr(),
                           0,
                           0,
                           0) == raw::Status::Err {
        return raw::Status::Err;
    }

    let rateNegativeCommand = RateNegativeCommand {};
    if raw::create_command(ctx,
                           format!("{}\0", rateNegativeCommand.name()).as_ptr(),
                           Some(RateNegative_RedisCommand),
                           format!("{}\0", rateNegativeCommand.str_flags()).as_ptr(),
                           0,
                           0,
                           0) == raw::Status::Err {
        return raw::Status::Err;
    }

    let rateNegativeCommand = CalculateRatingCommand {};
    if raw::create_command(ctx,
                           format!("{}\0", rateNegativeCommand.name()).as_ptr(),
                           Some(CalculateRating_RedisCommand),
                           format!("{}\0", rateNegativeCommand.str_flags()).as_ptr(),
                           0,
                           0,
                           0) == raw::Status::Err {
        return raw::Status::Err;
    }

    return raw::Status::Ok;
}

fn parse_i64(arg: &str) -> Result<i64, CellError> {
    arg.parse::<i64>()
        .map_err(|_| error!("Couldn't parse as integer: {}", arg))
}
