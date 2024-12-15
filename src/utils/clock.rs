//SPDX-FileCopyrightText: 2024 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use std::str::FromStr;
use ::time::Duration;
use nix::time;
use nix::time::ClockId;
use crate::core::data::variable::{Variable, Value};
use crate::core::data::variable::single::SingleData;

fn monotonic_time() -> Duration {
    let now = time::clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap();
    Duration::new(now.tv_sec(), now.tv_nsec() as i32)
}

pub fn set_seconds(v: &mut Variable, var: &str) -> String {
    let offset = Duration::seconds(i64::from_str(var).unwrap_or(0));
    let adjusted = monotonic_time() - offset;
    let text = format!("{}.{}", adjusted.whole_seconds(), adjusted.subsec_nanoseconds());
//    v.set_data(text);
    text
}

pub fn get_seconds(v: &mut Variable) -> Value {
    if let Value::Special(ref s) = v.value {
        let part: Vec<&str> = s.data.split('.').collect();
        let sec = i64::from_str(part[0]).unwrap();
        let nano = i32::from_str(part[1]).unwrap();
        let offset = Duration::new(sec, nano);
        let elapsed = monotonic_time() - offset;
        return Value::Single(SingleData::from(elapsed.whole_seconds().to_string()));
    }
    Value::None
}

pub fn get_epochseconds(_v: &mut Variable) -> Value {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    let epoch_seconds = real.tv_sec();
    Value::Single(SingleData::from(epoch_seconds.to_string()))
}

pub fn get_epochrealtime(_v: &mut Variable) -> Value {
    let real = time::clock_gettime(ClockId::CLOCK_REALTIME).unwrap();
    let epoch_realtime = format!("{}.{:06}", real.tv_sec(), real.tv_nsec() / 1000);
    Value::Single(SingleData::from(epoch_realtime))
}
